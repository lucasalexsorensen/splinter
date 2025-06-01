use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use esp_hal::i2c::master::{BusTimeout, Config as I2cConfig, I2c};
use esp_hal::mcpwm::operator::{PwmPin, PwmPinConfig};
use esp_hal::mcpwm::timer::PwmWorkingMode;
use esp_hal::mcpwm::{McPwm, PeripheralClockConfig};
use esp_hal::peripherals::{MCPWM0, RADIO_CLK, TIMG1, WIFI};
use esp_hal::rng::Rng;
use esp_hal::time::Rate;
use esp_hal::Async;
use esp_hal::{gpio::AnyPin, peripherals::Peripherals};
use static_cell::StaticCell;

pub type I2c0Bus = Mutex<NoopRawMutex, I2c<'static, Async>>;

pub struct WifiResources {
    pub rng: Rng,
    pub wifi: WIFI,
    pub wifi_clock: RADIO_CLK,
}

#[allow(dead_code)]
pub struct UltrasonicSensorResources {
    pub trigger_pin: AnyPin,
    pub echo_pin: AnyPin,
}

pub enum AnyPwmPin {
    LeftPin(PwmPin<'static, MCPWM0, 0, true>),
    RightPin(PwmPin<'static, MCPWM0, 0, false>),
}

pub struct MotorResources {
    pub pwm_pin: AnyPwmPin,
    pub forward_pin: AnyPin,
    pub backward_pin: AnyPin,
}

pub struct RotaryEncoderResources {
    pub encoder_a_pin: AnyPin,
    pub encoder_b_pin: AnyPin,
}

pub struct Resources {
    pub wifi: WifiResources,
    pub timer: TIMG1,
    pub i2c_bus: &'static I2c0Bus,
    pub left_motor: MotorResources,
    pub left_encoder: RotaryEncoderResources,
    pub right_motor: MotorResources,
    pub right_encoder: RotaryEncoderResources,
}

pub fn assign_resources(p: Peripherals) -> Resources {
    let i2c = I2c::new(
        p.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(400))
            .with_timeout(BusTimeout::Maximum),
    )
    .unwrap()
    .with_sda(p.GPIO21)
    .with_scl(p.GPIO22)
    .into_async();

    static I2C_BUS: StaticCell<I2c0Bus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(32)).unwrap();
    let mut mcpwm = McPwm::new(p.MCPWM0, clock_cfg);
    mcpwm.operator0.set_timer(&mcpwm.timer0);

    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, Rate::from_khz(5))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);
    let (left_pwm_pin, right_pwm_pin) = mcpwm.operator0.with_pins(
        p.GPIO13,
        PwmPinConfig::UP_ACTIVE_HIGH,
        p.GPIO25,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );

    let rng = Rng::new(p.RNG);

    Resources {
        wifi: WifiResources {
            rng,
            wifi: p.WIFI,
            wifi_clock: p.RADIO_CLK,
        },
        timer: p.TIMG1,
        i2c_bus,
        left_motor: MotorResources {
            pwm_pin: AnyPwmPin::LeftPin(left_pwm_pin),
            backward_pin: p.GPIO12.into(),
            forward_pin: p.GPIO14.into(),
        },
        left_encoder: RotaryEncoderResources {
            encoder_a_pin: p.GPIO32.into(),
            encoder_b_pin: p.GPIO33.into(),
        },
        right_motor: MotorResources {
            pwm_pin: AnyPwmPin::RightPin(right_pwm_pin),
            backward_pin: p.GPIO26.into(),
            forward_pin: p.GPIO27.into(),
        },
        right_encoder: RotaryEncoderResources {
            encoder_a_pin: p.GPIO34.into(),
            encoder_b_pin: p.GPIO35.into(),
        },
    }
}
