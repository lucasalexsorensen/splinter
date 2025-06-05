#![cfg(feature = "bluetooth")]

use core::sync::atomic::Ordering;

use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Timer};
use esp_println::println;

use trouble_host::prelude::*;

use crate::message::Message;
use crate::{command::Command, resources::BluetoothResources};
use esp_hal::timer::timg::Timer as HardwareTimer;
use esp_wifi_ble_only::ble::controller::BleConnector;
use esp_wifi_ble_only::{init, EspWifiController};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const CONNECTIONS_MAX: usize = 1;

const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

#[gatt_server]
struct Server {
    comm_service: CommunicationService,
}

#[gatt_service(uuid = "deadbeef-dead-beef-dead-beefdeadbeef")]
struct CommunicationService {
    /// TX Characteristic - for sending data to central
    #[characteristic(uuid = "deadbeef-dead-beef-dead-beefdeadbeef", notify)]
    tx: [u8; 20],
    /// RX Characteristic - for receiving data from central  
    #[characteristic(
        uuid = "deadbeef-dead-beef-dead-beefdeadbeed",
        write,
        write_without_response
    )]
    rx: [u8; 20],
}

#[embassy_executor::task]
pub async fn server_task(timer: HardwareTimer<'static>, resources: BluetoothResources) {
    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timer, resources.rng, resources.bt_clock).unwrap()
    );

    let connector = BleConnector::new(esp_wifi_ctrl, resources.bt);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Rat",
        appearance: &appearance::REMOTE_CONTROL,
    }))
    .unwrap();

    let _ = join(ble_task(runner), async {
        loop {
            match advertise("Rat", &mut peripheral, &server).await {
                Ok(conn) => {
                    let a = gatt_task(&server, &conn);
                    let b = tx_task(&server, &conn, &stack);
                    select(a, b).await;
                }
                Err(e) => {
                    println!("[adv] error: {:?}", e);
                }
            }
        }
    })
    .await;
}

async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            println!("[ble_task] error: {:?}", e);
        }
    }
}

async fn gatt_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let rx = server.comm_service.rx;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event: Err(e) } => {
                println!("[gatt] error processing event: {:?}", e)
            }
            GattConnectionEvent::Gatt { event: Ok(event) } => {
                match &event {
                    GattEvent::Read(event) => {
                        // TX cannot be read - only notified!
                        // if event.handle() == tx.handle {
                        //     let value = server.get(&tx);
                        //     println!("[gatt] Read Event to TX Characteristic: {:?}", value);
                        // }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == rx.handle {
                            println!(
                                "[gatt] Write Event to RX Characteristic: {:?}",
                                event.data()
                            );
                            process_rx_data(event.data());
                        }
                    }
                };
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => println!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {}
        }
    };
    println!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    println!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    println!("[adv] connection established");
    Ok(conn)
}

fn process_rx_data(data: &[u8]) {
    println!("[rx_handler] Received {} bytes: {:02X?}", data.len(), data);
}

async fn tx_task<C: Controller, P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    _stack: &Stack<'_, C, P>,
) {
    loop {
        let queue_future = crate::state::MESSAGE_QUEUE.receive();
        let timer_future = Timer::after(Duration::from_millis(50));

        let msg = match select(queue_future, timer_future).await {
            Either::First(msg) => msg,
            Either::Second(_) => Message::CountUpdated {
                left: crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed),
                right: crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed),
            },
        };

        let msg_bytes: [u8; 20] = msg.into();
        let notify_result = server.comm_service.tx.notify(conn, &msg_bytes).await;

        if notify_result.is_err() {
            break;
        };
    }
}
