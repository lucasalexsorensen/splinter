#![cfg(feature = "wifi")]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};

use embassy_net::tcp::TcpSocket;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::timer::timg::Timer as HardwareTimer;
use sha1::{Digest, Sha1};

use esp_println::println;
use esp_wifi::wifi::{
    AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use esp_wifi::{init, EspWifiController};

use crate::command::{Command, DisplayCommand};
use crate::config::BotConfig;
use crate::message::Message;
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
pub async fn server_task(
    spawner: Spawner,
    timer: HardwareTimer<'static>,
    mut resources: WifiResources,
) {
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
            crate::state::DISPLAY_COMMAND_QUEUE
                .send(DisplayCommand::IpChanged(config.address.address().octets()))
                .await;
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
            let incoming_future = socket.read(&mut buffer);
            let outgoing_future = crate::state::MESSAGE_QUEUE.receive();
            let timer_future = Timer::after(Duration::from_millis(50));

            match select3(incoming_future, outgoing_future, timer_future).await {
                Either3::First(read_result) => {
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
                Either3::Second(message) => {
                    handle_write(&mut socket, message).await;
                }
                Either3::Third(_) => {
                    handle_write(
                        &mut socket,
                        Message::CountUpdated {
                            left: crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed),
                            right: crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed),
                        },
                    )
                    .await;

                    handle_write(
                        &mut socket,
                        Message::GyroUpdated {
                            x: crate::state::GYRO_X.load(Ordering::Relaxed),
                            y: crate::state::GYRO_Y.load(Ordering::Relaxed),
                            z: crate::state::GYRO_Z.load(Ordering::Relaxed),
                        },
                    )
                    .await;
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

    // send our initial state by pushing some messages to the queue
    crate::state::MESSAGE_QUEUE
        .send(Message::CountUpdated {
            left: crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed),
            right: crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed),
        })
        .await;

    crate::state::MESSAGE_QUEUE
        .send(Message::TargetUpdated {
            left: crate::state::LEFT_ENCODER_TARGET.load(Ordering::Relaxed),
            right: crate::state::RIGHT_ENCODER_TARGET.load(Ordering::Relaxed),
        })
        .await;
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

    if unmasked_bytes[0] == 0x01 {
        crate::state::COMMAND_QUEUE.send(Command::TurnLeft).await;
    } else if unmasked_bytes[0] == 0x02 {
        crate::state::COMMAND_QUEUE.send(Command::TurnRight).await;
    } else if unmasked_bytes[0] == 0x03 {
        crate::state::COMMAND_QUEUE.send(Command::MoveForward).await;
    } else if unmasked_bytes[0] == 0x04 {
        crate::state::COMMAND_QUEUE
            .send(Command::MoveBackward)
            .await;
    } else if unmasked_bytes[0] == 0x05 {
        crate::state::COMMAND_QUEUE.send(Command::DebugMotors).await;
    } else if unmasked_bytes[0] == 0x06 {
        // read the next 4 bytes as f32, and the next 4 again as f32
        let k_p = f32::from_le_bytes([
            unmasked_bytes[1],
            unmasked_bytes[2],
            unmasked_bytes[3],
            unmasked_bytes[4],
        ]);
        let k_d = f32::from_le_bytes([
            unmasked_bytes[5],
            unmasked_bytes[6],
            unmasked_bytes[7],
            unmasked_bytes[8],
        ]);
        crate::state::COMMAND_QUEUE
            .send(Command::Configure(BotConfig { k_p, k_d }))
            .await;
    }
}

async fn handle_write(socket: &mut TcpSocket<'_>, message: Message) {
    let response_bytes: [u8; 20] = message.into();
    socket
        .write(&[0b10000010, response_bytes.len() as u8])
        .await
        .unwrap();
    socket.write(&response_bytes).await.unwrap();
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
