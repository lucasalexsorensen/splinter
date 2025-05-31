use crate::{
    resources::I2c0Bus,
    state::{LEFT_ENCODER_COUNT, RIGHT_ENCODER_COUNT},
};
use core::{fmt::Write, sync::atomic::Ordering};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306Async};

#[embassy_executor::task]
pub async fn display_task(i2c_bus: &'static I2c0Bus) {
    let device = I2cDevice::new(i2c_bus);
    let interface = I2CDisplayInterface::new(device);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().await.unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        display.clear(BinaryColor::Off).unwrap();
        let ip_address = crate::state::IP_ADDRESS.wait().await;

        Text::with_baseline(&ip_address, Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().await.unwrap();
    }
}
