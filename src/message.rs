use crate::config::BotConfig;

#[allow(clippy::enum_variant_names)]
pub enum Message {
    CountUpdated { left: i32, right: i32 },
    OrientationUpdated { yaw: i16, pitch: i16, roll: i16 },
    TargetUpdated { left: i32, right: i32 },
    ConfigUpdated(BotConfig),
    PIDDebug {},
}

impl From<Message> for [u8; 20] {
    fn from(val: Message) -> Self {
        let mut result = [0; 20];
        match val {
            Message::CountUpdated { left, right } => {
                result[0] = 0x01;
                result[1..5].copy_from_slice(&left.to_le_bytes());
                result[5..9].copy_from_slice(&right.to_le_bytes());
            }
            Message::TargetUpdated { left, right } => {
                result[0] = 0x02;
                result[1..5].copy_from_slice(&left.to_le_bytes());
                result[5..9].copy_from_slice(&right.to_le_bytes());
            }
            Message::OrientationUpdated { yaw, pitch, roll } => {
                result[0] = 0x03;
                result[1..3].copy_from_slice(&yaw.to_le_bytes());
                result[3..5].copy_from_slice(&pitch.to_le_bytes());
                result[5..7].copy_from_slice(&roll.to_le_bytes());
            }
            Message::ConfigUpdated(config) => {
                result[0] = 0x04;
                result[1..5].copy_from_slice(&config.k_p.to_le_bytes());
                result[5..9].copy_from_slice(&config.k_d.to_le_bytes());
            }
            Message::PIDDebug {} => {
                result[0] = 0x05;
            }
        }
        result
    }
}
