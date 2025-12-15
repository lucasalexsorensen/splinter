use core::sync::atomic::{AtomicI32, Ordering};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_println::println;

use crate::resources::{AnyPwmPin, MotorResources};

pub enum MotorSide {
    Left,
    Right,
}

pub struct DualMotorResources {
    pub left_motor: MotorResources,
    pub right_motor: MotorResources,
}

#[embassy_executor::task]
pub async fn dual_motor_task(mut resources: DualMotorResources) {
    // Initialize left motor pins
    let mut left_forward = Output::new(
        resources.left_motor.forward_pin,
        Level::Low,
        OutputConfig::default(),
    );
    let mut left_backward = Output::new(
        resources.left_motor.backward_pin,
        Level::Low,
        OutputConfig::default(),
    );

    // Initialize right motor pins
    let mut right_forward = Output::new(
        resources.right_motor.forward_pin,
        Level::Low,
        OutputConfig::default(),
    );
    let mut right_backward = Output::new(
        resources.right_motor.backward_pin,
        Level::Low,
        OutputConfig::default(),
    );

    let mut left_prev_error = 0;
    let mut right_prev_error = 0;
    let mut prev_sync_error = 0;
    let mut sync_integral = 0.0; // Track cumulative sync error for persistent issues

    loop {
        Timer::after(Duration::from_millis(50)).await;

        // Read current and target counts for both motors
        let left_current = crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed);
        let left_target = crate::state::LEFT_ENCODER_TARGET.load(Ordering::Relaxed);
        let right_current = crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed);
        let right_target = crate::state::RIGHT_ENCODER_TARGET.load(Ordering::Relaxed);

        // Calculate individual motor errors
        let left_error = left_target - left_current;
        let right_error = right_target - right_current;

        // Calculate synchronization error (difference between motor positions relative to their targets)
        // Positive sync_error means left motor is ahead of right motor relative to their respective targets
        let left_progress = if left_target != 0 {
            left_current as f32 / left_target as f32
        } else {
            0.0
        };
        let right_progress = if right_target != 0 {
            right_current as f32 / right_target as f32
        } else {
            0.0
        };
        let sync_error = ((left_progress - right_progress) * 1000.0) as i32; // Scale for better control

        // If both motors are close to target, stop both
        if left_error.abs() < 10 && right_error.abs() < 10 {
            stop_motor(&mut left_forward, &mut left_backward);
            stop_motor(&mut right_forward, &mut right_backward);
            left_prev_error = left_error;
            right_prev_error = right_error;
            prev_sync_error = sync_error;
            sync_integral *= 0.9; // Decay integral when stopped but don't reset completely
            continue;
        }

        // Calculate derivative terms
        let left_diff = left_error - left_prev_error;
        let right_diff = right_error - right_prev_error;
        let sync_diff = sync_error - prev_sync_error;

        // Load PID constants
        let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
        let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));

        // Cross-coupling gain (how much each motor responds to synchronization error)
        // Increased from 30% to 50% for stronger synchronization
        let k_sync = k_p * 0.5; // 50% of main proportional gain
        let k_sync_d = k_d * 0.5; // 50% of main derivative gain
        let k_sync_i = k_p * 0.1; // 10% integral gain for persistent sync errors

        // Update sync integral for persistent errors
        sync_integral += sync_error as f32 / 1000.0;
        sync_integral = sync_integral.clamp(-0.5, 0.5); // Prevent windup

        // Calculate base control values for both motors
        let left_base = calculate_control_value(left_error, left_diff, k_p, k_d);
        let right_base = calculate_control_value(right_error, right_diff, k_p, k_d);

        // Add cross-coupling terms with asymmetric correction
        // If left motor is ahead (positive sync_error), slow it down and speed up right
        // If right motor is ahead (negative sync_error), slow it down and speed up left
        let sync_p_term = k_sync * (sync_error as f32 / 1000.0);
        let sync_d_term = k_sync_d * (sync_diff as f32 / 1000.0);
        let sync_i_term = k_sync_i * sync_integral;
        let sync_correction = sync_p_term + sync_d_term + sync_i_term;

        // Apply asymmetric correction - if left motor consistently leads, apply stronger correction
        let left_correction_multiplier = if sync_error > 0 { 1.2 } else { 1.0 }; // 20% stronger when left leads
        let right_correction_multiplier = if sync_error < 0 { 1.2 } else { 1.0 }; // 20% stronger when right leads

        let left_val = (left_base - sync_correction * left_correction_multiplier).clamp(-1.0, 1.0);
        let right_val =
            (right_base + sync_correction * right_correction_multiplier).clamp(-1.0, 1.0);

        // Apply control to both motors simultaneously
        apply_motor_control(
            &mut left_forward,
            &mut left_backward,
            &mut resources.left_motor.pwm_pin,
            left_val,
        );

        apply_motor_control(
            &mut right_forward,
            &mut right_backward,
            &mut resources.right_motor.pwm_pin,
            right_val,
        );

        // Update previous errors
        left_prev_error = left_error;
        right_prev_error = right_error;
        prev_sync_error = sync_error;

        // Enhanced debug output for better synchronization monitoring
        if sync_error.abs() > 20 || (left_error.abs() > 5 || right_error.abs() > 5) {
            println!(
                "Sync: {} | L: {}/{} (err:{}, val:{:.2}) | R: {}/{} (err:{}, val:{:.2}) | Corr: {:.3}",
                sync_error, 
                left_current, left_target, left_error, left_val,
                right_current, right_target, right_error, right_val,
                sync_correction
            );
        }
    }
}

// Keep the original motor_task for backward compatibility if needed
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
        error = target_count - current_count;

        if error.abs() < 10 {
            backward.set_low();
            forward.set_low();
            continue;
        }

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

fn stop_motor(forward: &mut Output, backward: &mut Output) {
    forward.set_low();
    backward.set_low();
}

fn calculate_control_value(error: i32, diff: i32, k_p: f32, k_d: f32) -> f32 {
    let p_term = k_p * (error as f32);
    let d_term = k_d * (diff as f32);
    let val = p_term + d_term;
    val.clamp(-1.0, 1.0)
}

fn apply_motor_control(
    forward: &mut Output,
    backward: &mut Output,
    pwm_pin: &mut AnyPwmPin,
    val: f32,
) {
    if val.abs() < 0.01 {
        stop_motor(forward, backward);
        return;
    }

    if val.is_sign_positive() {
        forward.set_high();
        backward.set_low();
    } else {
        forward.set_low();
        backward.set_high();
    }

    set_pwm_strength(pwm_pin, val);
}

/// Strength should be a value between -1 and 1
fn set_pwm_strength(pwm_pin: &mut AnyPwmPin, strength: f32) {
    let val = (strength.abs() * 39.0_f32) as u16 + 60;
    match pwm_pin {
        AnyPwmPin::LeftPin(pin) => pin.set_timestamp(val),
        AnyPwmPin::RightPin(pin) => pin.set_timestamp(val),
    }
}
