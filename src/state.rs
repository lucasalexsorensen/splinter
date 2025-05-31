use core::sync::atomic::AtomicI32;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

// pub static LEFT_DISTANCE: Signal<CriticalSectionRawMutex, f64> = Signal::new();
// pub static RIGHT_DISTANCE: Signal<CriticalSectionRawMutex, f64> = Signal::new();

pub static LEFT_ENCODER_COUNT: AtomicI32 = AtomicI32::new(0);
pub static RIGHT_ENCODER_COUNT: AtomicI32 = AtomicI32::new(0);

pub static LEFT_ENCODER_TARGET: AtomicI32 = AtomicI32::new(0);
pub static RIGHT_ENCODER_TARGET: AtomicI32 = AtomicI32::new(0);

pub static IP_ADDRESS: Signal<CriticalSectionRawMutex, heapless::String<24>> = Signal::new();
