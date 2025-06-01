use core::sync::atomic::{AtomicI32, Ordering};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_println::println;

use crate::{
    command::MotorCommand,
    resources::{AnyPwmPin, MotorResources},
};

pub enum MotorSide {
    Left,
    Right,
}

#[embassy_executor::task(pool_size = 2)]
pub async fn motor_task(r: MotorResources, side: MotorSide) {
    let command_queue = match side {
        MotorSide::Left => &crate::state::LEFT_MOTOR_QUEUE,
        MotorSide::Right => &crate::state::RIGHT_MOTOR_QUEUE,
    };
    let encoder_count = match side {
        MotorSide::Left => &crate::state::LEFT_ENCODER_COUNT,
        MotorSide::Right => &crate::state::RIGHT_ENCODER_COUNT,
    };

    let forward = Output::new(r.forward_pin, Level::Low, OutputConfig::default());
    let backward = Output::new(r.backward_pin, Level::Low, OutputConfig::default());

    let mut pins = (forward, backward, r.pwm_pin);

    loop {
        let cmd = command_queue.receive().await;

        match cmd {
            MotorCommand::SetTarget(target) => {
                set_target(encoder_count, target, &mut pins).await;
            }
            MotorCommand::Debug(reversed) => {
                debug(&mut pins, reversed).await;
            }
        }
    }
}

async fn debug(pins: &mut (Output<'static>, Output<'static>, AnyPwmPin), reversed: bool) {
    let (forward, backward, pwm_pin) = pins;

    let pin = match reversed {
        true => forward,
        false => backward,
    };

    set_pwm(pwm_pin, 1.0);
    pin.set_high();
    Timer::after(Duration::from_millis(1000)).await;
    pin.set_low();
}

async fn set_target(
    encoder_count: &'static AtomicI32,
    target_count: i32,
    motor_pins: &mut (Output<'static>, Output<'static>, AnyPwmPin),
) {
    let (forward, backward, pwm_pin) = motor_pins;
    let mut count_within_threshold = 0;

    let mut error = 0;
    let mut last_error = 0;

    let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
    let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));

    loop {
        if count_within_threshold > 10 {
            forward.set_low();
            backward.set_low();
            break;
        }

        let current_count = encoder_count.load(Ordering::Relaxed);
        last_error = error;
        let error = target_count - current_count;

        if error.abs() < 10 {
            count_within_threshold += 1;
        }

        let derivative = error - last_error;

        let p_term = k_p * (error as f32);
        let d_term = k_d * (derivative as f32);

        let mut val = p_term + d_term;
        val = val.clamp(-1.0, 1.0);

        if val.is_sign_negative() {
            forward.set_high();
            backward.set_low();
        } else {
            forward.set_low();
            backward.set_high();
        }

        set_pwm(pwm_pin, val);
        Timer::after(Duration::from_millis(5)).await;
    }
}

/// Strenght should be a value between -1 and 1
fn set_pwm(pwm_pin: &mut AnyPwmPin, strength: f32) {
    let val = (strength.abs() * 39.0_f32) as u16 + 60;
    match pwm_pin {
        AnyPwmPin::LeftPin(pin) => pin.set_timestamp(val),
        AnyPwmPin::RightPin(pin) => pin.set_timestamp(val),
    }
}
