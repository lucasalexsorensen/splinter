use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use core::fmt::Write;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};

use embassy_net::tcp::TcpSocket;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::timer::timg::Timer as OtherTimer;
use sha1::{Digest, Sha1};

use esp_println::println;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use esp_wifi::{init, EspWifiController};

use crate::resources::WifiResources;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn connect_task(mut controller: WifiController<'static>) {
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller.start_async().await.unwrap();
        }

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi {:?}!", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn ws_server_task(spawner: Spawner, timer: OtherTimer, mut resources: WifiResources) {
    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timer, resources.rng, resources.wifi_clock).unwrap()
    );
    let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, resources.wifi).unwrap();
    let wifi_interface = interfaces.sta;
    let wifi_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = ((resources.rng.random() as u64) << 32) | resources.rng.random() as u64;
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        wifi_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connect_task(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            let mut ip_address = heapless::String::<24>::new();
            write!(ip_address, "{}", config.address).unwrap();
            crate::state::IP_ADDRESS.signal(ip_address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));
        socket.accept(9999).await.unwrap();

        let mut handshook = false;
        let mut buffer = [0; 1024];
        loop {
            buffer.fill(0);
            let read_future = socket.read(&mut buffer);
            let delay_future = Timer::after(Duration::from_millis(100));
            match select(read_future, delay_future).await {
                Either::First(read_result) => {
                    if let Ok(0) = read_result {
                        println!("Client disconnected");
                        break;
                    }
                    if !handshook {
                        handle_handshake(&buffer, &mut socket).await;
                        handshook = true;
                    } else {
                        handle_read_frame(&mut buffer, &mut socket).await;
                    }
                }
                Either::Second(_) => {
                    handle_write(&mut socket).await;
                }
            }
        }

        socket.abort();
        _ = socket.flush().await;
    }
}

async fn handle_handshake(buffer: &[u8], socket: &mut TcpSocket<'_>) {
    const NEEDLE: &[u8] = b"Sec-WebSocket-Key: ";
    let key_idx = find_subslice(buffer, NEEDLE);
    if key_idx.is_none() {
        return;
    }

    let start = key_idx.unwrap() + NEEDLE.len();
    let mut stop = start;
    while buffer[stop] != b'\r' && buffer[stop] != b'\n' {
        stop += 1;
    }

    let key = &buffer[start..stop];
    let accept_key = generate_accept_key(key);

    socket
                    .write(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ")
                    .await
                    .unwrap();
    socket.write(&accept_key).await.unwrap();
    socket.write(b"\r\n\r\n").await.unwrap();
}

async fn handle_read_frame(buffer: &mut [u8], socket: &mut TcpSocket<'_>) {
    // assert that the first bit is 1 (fin)
    let fin = buffer[0] & 0b10000000 == 0b10000000;
    if !fin {
        println!("Not a final frame");
        return;
    }
    let opcode = buffer[0] & 0b00001111;
    match opcode {
        // text and binary opcodes are fine
        0x1 | 0x2 => {}
        0x8 => {
            // send a close frame and return
            socket.write(&[0x88, 0x00]).await.unwrap();
            socket.flush().await.unwrap();
            return;
        }
        _ => {
            println!("Unknown opcode: {:x}", opcode);
            return;
        }
    }

    let is_masked = buffer[1] & 0b10000000 == 0b10000000;
    if !is_masked {
        println!("Not a masked message");
        return;
    }
    let payload_len = buffer[1] & 0b01111111;
    if payload_len > 125 {
        println!("Payload length too long");
        return;
    }
    for i in 0..payload_len as usize {
        buffer[6 + i] ^= buffer[2 + (i % 4)];
    }
    let unmasked_bytes = &buffer[6..6 + payload_len as usize];
    // let unmasked_str = core::str::from_utf8(unmasked_bytes).unwrap();
    // let unmasked_str: heapless::String<128> = unmasked_str.try_into().unwrap();
    // println!("Unmasked payload: {:?}", unmasked_str);

    const T: i32 = 3000;
    if unmasked_bytes[0] == 0x01 {
        // increase left target by T, decrease right target by T
        println!("turning left");
        crate::state::LEFT_ENCODER_TARGET.fetch_add(T, Ordering::Relaxed);
        crate::state::RIGHT_ENCODER_TARGET.fetch_sub(T, Ordering::Relaxed);
    } else if unmasked_bytes[0] == 0x02 {
        // decrease left target by T, increase right target by T
        println!("turning right");
        crate::state::LEFT_ENCODER_TARGET.fetch_sub(T, Ordering::Relaxed);
        crate::state::RIGHT_ENCODER_TARGET.fetch_add(T, Ordering::Relaxed);
    }
}

async fn handle_write(socket: &mut TcpSocket<'_>) {
    //let response_data: &[u8] = &[123];
    let left_count = crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed);
    let right_count = crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed);
    let left_target = crate::state::LEFT_ENCODER_TARGET.load(Ordering::Relaxed);
    let right_target = crate::state::RIGHT_ENCODER_TARGET.load(Ordering::Relaxed);

    // encode left count, right count as 4 bytes each
    let response_data = [
        left_count.to_le_bytes(),
        right_count.to_le_bytes(),
        left_target.to_le_bytes(),
        right_target.to_le_bytes(),
    ]
    .concat();

    socket
        // write a binary frame
        .write(&[0b10000010, response_data.len() as u8])
        .await
        .unwrap();
    socket.write(&response_data).await.unwrap();
    socket.flush().await.unwrap();
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn generate_accept_key(key: &[u8]) -> [u8; 28] {
    const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(key);
    hasher.update(GUID.as_bytes());
    let hash = hasher.finalize();

    let mut output = [0; 28];
    BASE64.encode_slice(hash, &mut output).unwrap();
    output
}
