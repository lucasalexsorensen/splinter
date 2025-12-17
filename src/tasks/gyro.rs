use core::sync::atomic::Ordering;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Delay, Duration, Timer};
use esp_println::println;
use mpu6050_dmp::{
    address::Address, quaternion::Quaternion, sensor_async::Mpu6050, yaw_pitch_roll::YawPitchRoll,
};

use crate::resources::I2c0Bus;

#[embassy_executor::task]
pub async fn gyro_task(i2c_bus: &'static I2c0Bus) {
    let device = I2cDevice::new(i2c_bus);
    let mut mpu = Mpu6050::new(device, Address::default()).await.unwrap();

    println!("initializing DMP");
    mpu.initialize_dmp(&mut Delay).await.unwrap();
    println!("DMP initialized");

    // Configure sample rate
    mpu.set_sample_rate_divider(9).await.unwrap(); // 100Hz
    println!("sample rate configured");

    // Enable FIFO for quaternion data
    mpu.enable_fifo().await.unwrap();
    println!("FIFO enabled");

    // Buffer for FIFO data (DMP packets are 28 bytes)
    let mut buffer = [0u8; 28];

    loop {
        let fifo_count = mpu.get_fifo_count().await.unwrap();

        if fifo_count >= 28 {
            // Read a complete DMP packet
            let data = mpu.read_fifo(&mut buffer).await.unwrap();

            // First 16 bytes contain quaternion data
            let quat = Quaternion::from_bytes(&data[..16]).unwrap().normalize();

            // Convert quaternion to yaw, pitch, roll (radians)
            let ypr = YawPitchRoll::from(quat);

            // Convert to degrees and scale to i16 (multiply by 100 for 2 decimal precision)
            let yaw_deg = (ypr.yaw * 180.0 / core::f32::consts::PI * 100.0) as i16;
            let pitch_deg = (ypr.pitch * 180.0 / core::f32::consts::PI * 100.0) as i16;
            let roll_deg = (ypr.roll * 180.0 / core::f32::consts::PI * 100.0) as i16;

            crate::state::YAW.store(yaw_deg, Ordering::Relaxed);
            crate::state::PITCH.store(pitch_deg, Ordering::Relaxed);
            crate::state::ROLL.store(roll_deg, Ordering::Relaxed);
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}
