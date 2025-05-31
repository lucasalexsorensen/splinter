#![no_std]
#![no_main]
mod resources;
mod state;
mod tasks;

use embassy_executor::Spawner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt;
use esp_hal::peripherals::Interrupt;
use esp_hal::timer::timg::TimerGroup;
use tasks::{display, gyro, motor, net, rotary};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let resources = resources::assign_resources(p);

    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority1).unwrap();

    let timer = TimerGroup::new(resources.timer);
    esp_hal_embassy::init(timer.timer0);

    spawner
        .spawn(rotary::rotary_task(
            resources.left_encoder,
            &crate::state::LEFT_ENCODER_COUNT,
            true,
        ))
        .unwrap();
    spawner
        .spawn(rotary::rotary_task(
            resources.right_encoder,
            &crate::state::RIGHT_ENCODER_COUNT,
            false,
        ))
        .unwrap();
    // spawner.spawn(gyro::gyro_task(resources.i2c_bus)).unwrap();

    spawner
        .spawn(motor::motor_task(
            resources.left_motor,
            &crate::state::LEFT_ENCODER_COUNT,
            &crate::state::LEFT_ENCODER_TARGET,
        ))
        .unwrap();
    spawner
        .spawn(motor::motor_task(
            resources.right_motor,
            &crate::state::RIGHT_ENCODER_COUNT,
            &crate::state::RIGHT_ENCODER_TARGET,
        ))
        .unwrap();

    spawner
        .spawn(display::display_task(resources.i2c_bus))
        .unwrap();

    spawner
        .spawn(net::ws_server_task(spawner, timer.timer1, resources.wifi))
        .unwrap();
}
