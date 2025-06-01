use crate::command::Command;
use heapless::Vec;

pub enum Message {
    CountUpdated { left: i32, right: i32 },
    TargetUpdated { left: i32, right: i32 },
    QueueUpdated { commands: Vec<Command, 5> },
}
