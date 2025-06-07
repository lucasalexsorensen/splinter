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
    let mut current_speed = 0.0f32; // Current actual motor speed (-1.0 to 1.0)

    // Acceleration/deceleration parameters
    const MAX_ACCELERATION: f32 = 0.15; // Maximum change in speed per 50ms cycle
    const MIN_SPEED_THRESHOLD: f32 = 0.05; // Minimum speed before stopping completely

    loop {
        Timer::after(Duration::from_millis(50)).await;

        let current_count = current_count_signal.load(Ordering::Relaxed);
        let target_count = target_count_signal.load(Ordering::Relaxed);

        prev_error = error;
        // Error is the difference between the target count and the current count
        // positive => we need to go forward to increase the count
        // negative => we need to go backward to decrease the count
        error = target_count - current_count;

        let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
        let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));

        let p_term = k_p * (error as f32);
        let d_term = k_d * ((error - prev_error) as f32);

        let mut target_speed = p_term + d_term;
        target_speed = target_speed.clamp(-1.0, 1.0);

        // If we're close to target, gradually slow down to zero
        if error.abs() < 10 {
            target_speed = 0.0;
        }

        // Apply acceleration/deceleration ramping
        let speed_diff = target_speed - current_speed;
        let max_change = if speed_diff.abs() <= MAX_ACCELERATION {
            speed_diff // Can reach target speed this cycle
        } else {
            MAX_ACCELERATION * speed_diff.signum() // Limit to max acceleration
        };

        current_speed += max_change;

        // Apply minimum speed threshold - if speed is very low, just stop
        if current_speed.abs() < MIN_SPEED_THRESHOLD {
            current_speed = 0.0;
        }

        // Set motor direction and PWM based on current_speed
        if current_speed.abs() < 0.01 {
            forward.set_low();
            backward.set_low();
        } else if current_speed > 0.0 {
            forward.set_high();
            backward.set_low();
            set_pwm_strength(&mut r.pwm_pin, current_speed);
        } else {
            forward.set_low();
            backward.set_high();
            set_pwm_strength(&mut r.pwm_pin, current_speed);
        }
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
