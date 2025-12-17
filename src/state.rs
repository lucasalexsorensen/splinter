use core::sync::atomic::{AtomicI16, AtomicI32, AtomicU32};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::command::{Command, DisplayCommand};
use crate::message::Message;

// pub static LEFT_DISTANCE: Signal<CriticalSectionRawMutex, f64> = Signal::new();
// pub static RIGHT_DISTANCE: Signal<CriticalSectionRawMutex, f64> = Signal::new();

pub static LEFT_ENCODER_COUNT: AtomicI32 = AtomicI32::new(0);
pub static RIGHT_ENCODER_COUNT: AtomicI32 = AtomicI32::new(0);

// MPU6050 orientation data (yaw/pitch/roll in degrees * 100 for 2 decimal precision)
pub static YAW: AtomicI16 = AtomicI16::new(0);
pub static PITCH: AtomicI16 = AtomicI16::new(0);
pub static ROLL: AtomicI16 = AtomicI16::new(0);

pub static LEFT_ENCODER_TARGET: AtomicI32 = AtomicI32::new(0);
pub static RIGHT_ENCODER_TARGET: AtomicI32 = AtomicI32::new(0);

// pub static IP_ADDRESS: Signal<CriticalSectionRawMutex, heapless::String<24>> = Signal::new();
// pub static CONFIG: Mutex<CriticalSectionRawMutex, BotConfig> = Mutex::new(BotConfig::default());
pub static K_P: AtomicU32 = AtomicU32::new(0.003f32.to_bits());
pub static K_D: AtomicU32 = AtomicU32::new(0.006f32.to_bits());

pub type CommandQueue = Channel<CriticalSectionRawMutex, Command, 5>;
pub static COMMAND_QUEUE: CommandQueue = Channel::new();
// pub type MotorCommandQueue = Channel<CriticalSectionRawMutex, MotorCommand, 5>;
// pub static LEFT_MOTOR_QUEUE: MotorCommandQueue = Channel::new();
// pub static RIGHT_MOTOR_QUEUE: MotorCommandQueue = Channel::new();
type MessageQueue = Channel<CriticalSectionRawMutex, Message, 5>;
pub static MESSAGE_QUEUE: MessageQueue = Channel::new();

type DisplayCommandQueue = Channel<CriticalSectionRawMutex, DisplayCommand, 5>;
pub static DISPLAY_COMMAND_QUEUE: DisplayCommandQueue = Channel::new();
