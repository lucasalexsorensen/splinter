use defmt::info;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Delay, Duration, Instant, Timer};
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig};
use hcsr04_async::{Config, DistanceUnit, Hcsr04, Now, TemperatureUnit};

use crate::resources::UltrasonicSensorResources;

#[embassy_executor::task(pool_size = 2)]
pub async fn ultrasonic_task(
    r: UltrasonicSensorResources,
    signal: &'static Signal<CriticalSectionRawMutex, f64>,
) {
    let trigger_pin = Output::new(r.trigger_pin, Level::Low, OutputConfig::default());
    let echo_pin = Input::new(r.echo_pin, InputConfig::default());

    let config = Config {
        distance_unit: DistanceUnit::Centimeters,
        temperature_unit: TemperatureUnit::Celsius,
    };
    struct EmbassyClock;

    impl Now for EmbassyClock {
        fn now_micros(&self) -> u64 {
            Instant::now().as_micros()
        }
    }

    let clock = EmbassyClock;
    let delay = Delay;
    let mut sensor = Hcsr04::new(trigger_pin, echo_pin, config, clock, delay);
    let temperature = 24.0;

    loop {
        let distance = sensor.measure(temperature).await;
        match distance {
            Ok(distance) => {
                signal.signal(distance);
            }
            Err(e) => {
                info!("Error: {:?}", e);
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
