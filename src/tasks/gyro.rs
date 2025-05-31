use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Delay, Duration, Timer};
use mpu6050_async::Mpu6050;

use crate::resources::I2c0Bus;

#[embassy_executor::task]
pub async fn gyro_task(i2c_bus: &'static I2c0Bus) {
    let device = I2cDevice::new(i2c_bus);
    let mut mpu = Mpu6050::new(device);
    match mpu.init(&mut Delay).await {
        Ok(_) => info!("MPU6050 initialized"),
        Err(e) => match e {
            mpu6050_async::Mpu6050Error::I2c(_e) => info!("I2C error"),
            mpu6050_async::Mpu6050Error::InvalidChipId(e) => info!("Invalid chip ID: {:?}", e),
        },
    }

    loop {
        let temp = mpu.get_temp().await.unwrap();
        info!("temp: {:?}c", temp);
        Timer::after(Duration::from_millis(500)).await;
    }
}
