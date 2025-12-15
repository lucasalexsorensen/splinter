#![no_std]
#![no_main]

use anyhow::Result;

mod command;
mod config;
mod message;
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
use tasks::{
    display, gyro,
    motor::{self, DualMotorResources, MotorSide},
    orchestrator, rotary,
};

#[cfg(feature = "wifi")]
use tasks::wifi;

#[cfg(feature = "bluetooth")]
use tasks::bluetooth;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    spawn_tasks(spawner).await.expect("Failed to spawn tasks");
}

async fn spawn_tasks(spawner: Spawner) -> Result<()> {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let resources = resources::assign_resources(p);

    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority1)
        .map_err(|e| anyhow::anyhow!("Failed to enable GPIO interrupt: {:?}", e))?;

    let timer = TimerGroup::new(resources.timer);
    esp_hal_embassy::init(timer.timer0);

    spawner.spawn(display::display_task(resources.i2c_bus))?;

    #[cfg(feature = "wifi")]
    spawner.spawn(wifi::server_task(spawner, timer.timer1, resources.wifi))?;

    #[cfg(feature = "bluetooth")]
    spawner.spawn(bluetooth::server_task(timer.timer1, resources.bluetooth))?;

    spawner.spawn(rotary::rotary_task(
        resources.left_encoder,
        &crate::state::LEFT_ENCODER_COUNT,
        true,
    ))?;
    spawner.spawn(rotary::rotary_task(
        resources.right_encoder,
        &crate::state::RIGHT_ENCODER_COUNT,
        false,
    ))?;
    spawner.spawn(gyro::gyro_task(resources.i2c_bus))?;

    // Use dual motor task for synchronized control
    let dual_motor_resources = DualMotorResources {
        left_motor: resources.left_motor,
        right_motor: resources.right_motor,
    };
    spawner.spawn(motor::dual_motor_task(dual_motor_resources))?;

    spawner.spawn(orchestrator::orchestrator_task())?;

    Ok(())
}
