use crate::config::BotConfig;

#[allow(clippy::enum_variant_names)]
pub enum Message {
    CountUpdated { left: i32, right: i32 },
    GyroUpdated { x: i16, y: i16, z: i16 },
    TargetUpdated { left: i32, right: i32 },
    ConfigUpdated(BotConfig),
}

impl Into<[u8; 20]> for Message {
    fn into(self) -> [u8; 20] {
        let mut result = [0; 20];
        match self {
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
            Message::GyroUpdated { x, y, z } => {
                result[0] = 0x03;
                result[1..5].copy_from_slice(&x.to_le_bytes());
                result[5..9].copy_from_slice(&y.to_le_bytes());
                result[9..13].copy_from_slice(&z.to_le_bytes());
            }
            Message::ConfigUpdated(config) => {
                result[0] = 0x04;
                result[1..5].copy_from_slice(&config.k_p.to_le_bytes());
                result[5..9].copy_from_slice(&config.k_d.to_le_bytes());
            }
        }
        result
    }
}
