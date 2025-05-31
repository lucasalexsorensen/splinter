use core::sync::atomic::{AtomicI32, Ordering};

use embassy_futures::select::select;
use esp_hal::gpio::{Input, InputConfig};
use rotary_encoder_hal::Rotary;

use crate::resources::RotaryEncoderResources;

#[embassy_executor::task(pool_size = 2)]
pub async fn rotary_task(r: RotaryEncoderResources, count: &'static AtomicI32, flipped: bool) {
    let mut a_pin = Input::new(r.encoder_a_pin, InputConfig::default());
    let mut b_pin = Input::new(r.encoder_b_pin, InputConfig::default());
    let mut rotary = Rotary::new(&mut a_pin, &mut b_pin);
    loop {
        let (pin_a, pin_b) = rotary.pins();
        select(pin_a.wait_for_any_edge(), pin_b.wait_for_any_edge()).await;
        let direction = rotary.update().unwrap();
        let c = match direction {
            rotary_encoder_hal::Direction::Clockwise => 1,
            rotary_encoder_hal::Direction::CounterClockwise => -1,
            _ => 0,
        };
        if flipped {
            count.fetch_sub(c, Ordering::Relaxed);
        } else {
            count.fetch_add(c, Ordering::Relaxed);
        }
    }
}
