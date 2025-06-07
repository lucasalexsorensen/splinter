use core::sync::atomic::Ordering;

use crate::{
    command::{Command, DisplayCommand},
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
    crate::state::LEFT_ENCODER_TARGET.store(desired_left_count, Ordering::Relaxed);
    crate::state::RIGHT_ENCODER_TARGET.store(desired_right_count, Ordering::Relaxed);
    crate::state::MESSAGE_QUEUE
        .send(Message::TargetUpdated {
            left: desired_left_count,
            right: desired_right_count,
        })
        .await;
}
