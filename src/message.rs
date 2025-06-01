use crate::command::Command;
use heapless::Vec;

pub enum Message {
    CountUpdated {
        left: i32,
        right: i32,
    },
    GyroUpdated {
        x: i16,
        y: i16,
        z: i16,
    },
    TargetUpdated {
        left: i32,
        right: i32,
    },
    #[allow(dead_code)]
    QueueUpdated {
        commands: Vec<Command, 5>,
    },
}
