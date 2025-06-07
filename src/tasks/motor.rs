use core::sync::atomic::{AtomicI32, Ordering};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_println::println;

use crate::resources::{AnyPwmPin, MotorResources};

pub enum MotorSide {
    Left,
    Right,
}

#[embassy_executor::task(pool_size = 2)]
pub async fn motor_task(mut r: MotorResources, side: MotorSide) {
    let mut forward = Output::new(r.forward_pin, Level::Low, OutputConfig::default());
    let mut backward = Output::new(r.backward_pin, Level::Low, OutputConfig::default());

    let current_count_signal = match side {
        MotorSide::Left => &crate::state::LEFT_ENCODER_COUNT,
        MotorSide::Right => &crate::state::RIGHT_ENCODER_COUNT,
    };

    let target_count_signal = match side {
        MotorSide::Left => &crate::state::LEFT_ENCODER_TARGET,
        MotorSide::Right => &crate::state::RIGHT_ENCODER_TARGET,
    };

    let mut prev_error = 0;
    let mut error = 0;
    loop {
        Timer::after(Duration::from_millis(50)).await;

        let current_count = current_count_signal.load(Ordering::Relaxed);
        let target_count = target_count_signal.load(Ordering::Relaxed);

        prev_error = error;
        // Error is the difference between the target count and the current count
        // positive => we need to go forward to increase the count
        // negative => we need to go backward to decrease the count
        error = target_count - current_count;

        if error.abs() < 10 {
            backward.set_low();
            forward.set_low();
            continue;
        }

        // Derivative is the difference between the current error and the previous error
        // positive => the error has increased since last time
        // negative => the error has decreased since last time
        let diff = error - prev_error;

        let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
        let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));

        let p_term = k_p * (error as f32);
        let d_term = k_d * (diff as f32);

        let mut val = p_term + d_term;
        val = val.clamp(-1.0, 1.0);

        if val.abs() < 0.01 {
            forward.set_low();
            backward.set_low();
            continue;
        }

        if val.is_sign_positive() {
            forward.set_high();
            backward.set_low();
        } else {
            forward.set_low();
            backward.set_high();
        }
        set_pwm_strength(&mut r.pwm_pin, val);
    }
}

/// Strength should be a value between -1 and 1
fn set_pwm_strength(pwm_pin: &mut AnyPwmPin, strength: f32) {
    let val = (strength.abs() * 39.0_f32) as u16 + 60;
    match pwm_pin {
        AnyPwmPin::LeftPin(pin) => pin.set_timestamp(val),
        AnyPwmPin::RightPin(pin) => pin.set_timestamp(val),
    }
}
