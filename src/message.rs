use crate::command::Command;
use heapless::Vec;

#[allow(clippy::enum_variant_names)]
pub enum Message {
    CountUpdated { left: i32, right: i32 },
    GyroUpdated { x: i16, y: i16, z: i16 },
    TargetUpdated { left: i32, right: i32 },
}

impl Into<[u8; 20]> for Message {
    fn into(self) -> [u8; 20] {
        let mut result = [0; 20];
        match self {
            Message::CountUpdated { left, right } => {
                result[0] = 0x01;
                let left_bytes = left.to_le_bytes();
                let right_bytes = right.to_le_bytes();
                result[1..5].copy_from_slice(&left_bytes);
                result[5..9].copy_from_slice(&right_bytes);
            }
            Message::TargetUpdated { left, right } => {
                result[0] = 0x02;
                let left_bytes = left.to_le_bytes();
                let right_bytes = right.to_le_bytes();
                result[1..5].copy_from_slice(&left_bytes);
                result[5..9].copy_from_slice(&right_bytes);
            }
            Message::GyroUpdated { x, y, z } => {
                result[0] = 0x03;
                let x_bytes = x.to_le_bytes();
                let y_bytes = y.to_le_bytes();
                let z_bytes = z.to_le_bytes();
                result[1..5].copy_from_slice(&x_bytes);
                result[5..9].copy_from_slice(&y_bytes);
                result[9..13].copy_from_slice(&z_bytes);
            }
        }

        result
    }
}
