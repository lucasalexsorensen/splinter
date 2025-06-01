use crate::{command::DisplayCommand, resources::I2c0Bus};
use core::{fmt::Write, sync::atomic::Ordering};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
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
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    display.init().await.unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    display.clear(BinaryColor::Off).unwrap();

    // strings to display
    let mut ip_address_str: heapless::String<30> = heapless::String::new();
    let mut k_p_str: heapless::String<30> = heapless::String::new();
    let mut k_d_str: heapless::String<30> = heapless::String::new();

    write!(ip_address_str, "NO IP ADDRESS").unwrap();
    let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
    let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));
    write!(k_p_str, "K_P={:.4}", k_p).unwrap();
    write!(k_d_str, "K_D={:.4}", k_d).unwrap();

    loop {
        let display_cmd = crate::state::DISPLAY_COMMAND_QUEUE.receive().await;
        display.clear(BinaryColor::Off).unwrap();
        match display_cmd {
            DisplayCommand::IpChanged(ip_address) => {
                ip_address_str.clear();
                write!(
                    ip_address_str,
                    "{}.{}.{}.{}",
                    ip_address[0], ip_address[1], ip_address[2], ip_address[3]
                )
                .unwrap();
            }
            DisplayCommand::ConfigChanged => {
                k_p_str.clear();
                k_d_str.clear();
                let k_p = f32::from_bits(crate::state::K_P.load(Ordering::Relaxed));
                let k_d = f32::from_bits(crate::state::K_D.load(Ordering::Relaxed));
                write!(k_p_str, "K_P={:.4}", k_p).unwrap();
                write!(k_d_str, "K_D={:.4}", k_d).unwrap();
            }
        }

        Text::with_baseline(&ip_address_str, Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&k_p_str, Point::new(0, 10), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&k_d_str, Point::new(0, 20), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().await.unwrap();
    }
}
