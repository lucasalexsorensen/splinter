use core::sync::atomic::Ordering;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Delay, Duration, Timer};
use esp_println::println;
use mpu6050_dmp::{address::Address, sensor_async::Mpu6050};

use crate::resources::I2c0Bus;

#[embassy_executor::task]
pub async fn gyro_task(i2c_bus: &'static I2c0Bus) {
    let device = I2cDevice::new(i2c_bus);
    let mut mpu = Mpu6050::new(device, Address::default()).await.unwrap();

    // println!("initializing DMP");
    mpu.initialize_dmp(&mut Delay).await.unwrap();

    // println!("calibrating");
    // let calibration_params = CalibrationParameters::new(
    //     mpu6050_dmp::accel::AccelFullScale::G2,
    //     mpu6050_dmp::gyro::GyroFullScale::Deg2000,
    //     mpu6050_dmp::calibration::ReferenceGravity::ZN,
    // );
    // match mpu.calibrate(&mut Delay, &calibration_params).await {
    //     Ok(_) => println!("calibrated"),
    //     Err(e) => println!("calibration failed: {:?}", e),
    // }
    // println!("calibrated");

    loop {
        let gyro = mpu.gyro().await.unwrap();
        crate::state::GYRO_X.store(gyro.x(), Ordering::Relaxed);
        crate::state::GYRO_Y.store(gyro.y(), Ordering::Relaxed);
        crate::state::GYRO_Z.store(gyro.z(), Ordering::Relaxed);
        Timer::after(Duration::from_millis(50)).await;
    }
}
