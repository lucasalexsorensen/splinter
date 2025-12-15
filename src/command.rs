use crate::config::BotConfig;

#[derive(Debug)]
pub enum Command {
    TurnLeft,
    TurnRight,
    MoveForward,
    MoveBackward,
    DebugMotors,
    Configure(BotConfig),
}

impl From<&[u8]> for Command {
    fn from(value: &[u8]) -> Self {
        match value[0] {
            0x01 => Command::TurnLeft,
            0x02 => Command::TurnRight,
            0x03 => Command::MoveForward,
            0x04 => Command::MoveBackward,
            0x05 => Command::DebugMotors,
            0x06 => {
                let k_p = f32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let k_d = f32::from_le_bytes([value[5], value[6], value[7], value[8]]);
                Command::Configure(BotConfig { k_p, k_d })
            }
            _ => panic!("Unknown command: {:?}", value),
        }
    }
}

// #[derive(Debug)]
// pub enum MotorCommand {
//     SetTarget(i32),
//     Debug(bool),
// }

#[derive(Debug)]
pub enum DisplayCommand {
    BTConnected,
    BTDisconnected,
    ConfigChanged,
}
