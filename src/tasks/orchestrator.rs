use core::sync::atomic::Ordering;

use esp_println::println;

use crate::{
    command::{Command, DisplayCommand, MotorCommand},
    message::Message,
};

#[embassy_executor::task]
pub async fn orchestrator_task() {
    let mut flip = false;
    loop {
        let command = crate::state::COMMAND_QUEUE.receive().await;

        let current_left_count = crate::state::LEFT_ENCODER_COUNT.load(Ordering::Relaxed);
        let current_right_count = crate::state::RIGHT_ENCODER_COUNT.load(Ordering::Relaxed);

        const N_TURN: i32 = 1600;
        const N_STEP: i32 = 5000;
        match command {
            Command::TurnLeft => {
                handle_movement(current_left_count - N_TURN, current_right_count + N_TURN).await;
            }
            Command::TurnRight => {
                handle_movement(current_left_count + N_TURN, current_right_count - N_TURN).await;
            }
            Command::MoveForward => {
                handle_movement(current_left_count + N_STEP, current_right_count + N_STEP).await;
            }
            Command::MoveBackward => {
                handle_movement(current_left_count - N_STEP, current_right_count - N_STEP).await;
            }
            Command::DebugMotors => {
                crate::state::LEFT_MOTOR_QUEUE
                    .send(MotorCommand::Debug(flip))
                    .await;
                crate::state::RIGHT_MOTOR_QUEUE
                    .send(MotorCommand::Debug(flip))
                    .await;
                flip = !flip;
                continue;
            }
            Command::Configure(config) => {
                crate::state::K_P.store(config.k_p.to_bits(), Ordering::Relaxed);
                crate::state::K_D.store(config.k_d.to_bits(), Ordering::Relaxed);
                crate::state::DISPLAY_COMMAND_QUEUE
                    .send(DisplayCommand::ConfigChanged)
                    .await;
            }
        };
    }
}

async fn handle_movement(desired_left_count: i32, desired_right_count: i32) {
    crate::state::LEFT_MOTOR_QUEUE
        .send(MotorCommand::SetTarget(desired_left_count))
        .await;
    crate::state::RIGHT_MOTOR_QUEUE
        .send(MotorCommand::SetTarget(desired_right_count))
        .await;
    crate::state::MESSAGE_QUEUE
        .send(Message::TargetUpdated {
            left: desired_left_count,
            right: desired_right_count,
        })
        .await;
}
