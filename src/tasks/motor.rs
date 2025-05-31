use core::sync::atomic::{AtomicI32, Ordering};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};

use crate::resources::{AnyPwmPin, MotorResources};

#[embassy_executor::task(pool_size = 2)]
pub async fn motor_task(
    mut r: MotorResources,
    encoder_count: &'static AtomicI32,
    target_count: &'static AtomicI32,
) {
    let mut forward = Output::new(r.forward_pin, Level::Low, OutputConfig::default());
    let mut backward = Output::new(r.backward_pin, Level::Low, OutputConfig::default());

    let mut error = 0;
    let mut last_error;

    loop {
        let current_count = encoder_count.load(Ordering::Relaxed);
        let target_count = target_count.load(Ordering::Relaxed);
        last_error = error;
        error = target_count - current_count;

        let kp = 0.05;
        let kd = 0.001;
        let derivative = error - last_error;

        let mut val = kp * (error as f64) + kd * (derivative as f64);
        val = val.clamp(-1.0, 1.0);

        if val.is_sign_negative() {
            forward.set_high();
            backward.set_low();
        } else {
            forward.set_low();
            backward.set_high();
        }

        // map from 0-1 to 50-99
        let strength = map_to_strength(val);
        match &mut r.pwm_pin {
            AnyPwmPin::LeftPin(pin) => pin.set_timestamp(strength),
            AnyPwmPin::RightPin(pin) => pin.set_timestamp(strength),
        }

        Timer::after(Duration::from_millis(5)).await;
    }
}

fn map_to_strength(val: f64) -> u16 {
    // map from 0-1 to 50-99
    
    (val.abs() * 49.0_f64) as u16 + 50
}
