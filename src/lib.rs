#![no_std]

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::spi::{Mode, MODE_3};

pub mod bus;
pub mod measurement;
#[macro_use]
pub mod registers;

use bus::RegAccess;
pub use measurement::{Acceleration, Gyro, Temperature};
use registers::*;

pub enum IntPinConfig {
    IntReadClear = 4,
}

pub enum Interrupt {
    DataReady = 0,
}

pub enum ClockSource {
    Internal = 0,
    PLLGyroX = 1,
    PLLGyroY = 2,
    PLLGyroZ = 3,
    PLLExternal32_768KHz = 4,
    PLLExternal19_2MHz = 5,
    Stop = 7,
}

pub const SPI_MODE: Mode = MODE_3;

#[derive(Default)]
pub struct FifoEnable {
    pub temperature: bool,
    pub x_g_force: bool,
    pub y_g_force: bool,
    pub z_g_force: bool,
    pub acceleration: bool,
    pub slave2: bool,
    pub slave1: bool,
    pub slave0: bool,
}

impl Into<u8> for FifoEnable {
    fn into(self) -> u8 {
        (self.temperature as u8) << 7
            | (self.x_g_force as u8) << 6
            | (self.y_g_force as u8) << 5
            | (self.z_g_force as u8) << 4
            | (self.acceleration as u8) << 3
            | (self.slave2 as u8) << 2
            | (self.slave1 as u8) << 1
            | (self.slave0 as u8) << 0
    }
}

pub struct MPU6000<BUS> {
    bus: BUS,
    dlpf_enabled: bool,
    whoami: u8,
}

impl<E, BUS: RegAccess<Error = E>> MPU6000<BUS> {
    pub fn new(bus: BUS) -> Self {
        MPU6000 { bus, dlpf_enabled: false, whoami: 0x68 }
    }

    pub fn set_register(&mut self, reg: Register, offset: u8, len: u8, bits: u8) -> Result<(), E> {
        let mut value = self.bus.read(reg)?;
        let mask = (1u8 << len) - 1;
        value &= !(mask << offset);
        value |= (bits & mask) << offset;
        self.bus.write(reg, value)
    }

    pub fn set_slave_address(&mut self, address: u8) {
        self.whoami = address
    }

    pub fn whoami(&mut self) -> Result<u8, E> {
        self.bus.read(Register::WhoAmI)
    }

    pub fn product_id(&mut self) -> Result<u8, E> {
        self.bus.read(Register::ProductId)
    }

    pub fn verify(&mut self) -> Result<bool, E> {
        Ok(self.whoami()? == self.whoami && self.product_id()? != ProductId::Unknown as u8)
    }

    /// Required when connected via BUS
    pub fn reset<D: DelayMs<u8>>(&mut self, delay: &mut D) -> Result<(), E> {
        let reset_bit = PowerManagement1::DeviceReset as u8;
        self.bus.write(Register::PowerManagement1, reset_bit)?;
        delay.delay_ms(150u8.into());

        let value = SignalPathReset::TemperatureReset as u8
            | SignalPathReset::AccelerometerReset as u8
            | SignalPathReset::GyroReset as u8;
        self.bus.write(Register::SignalPathReset, value)?;
        delay.delay_ms(150u8.into());
        Ok(())
    }

    pub fn set_sleep(&mut self, enable: bool) -> Result<(), E> {
        self.set_register(Register::PowerManagement1, 6, 1, enable as u8)?;
        Ok(())
    }

    pub fn set_clock_source(&mut self, source: ClockSource) -> Result<(), E> {
        self.set_register(Register::PowerManagement1, 0, 3, source as u8)
    }

    pub fn set_dlpf(&mut self, value: u8) -> Result<(), E> {
        self.dlpf_enabled = 0 < value && value < 7;
        self.set_register(Register::Configuration, 0, 3, value as u8)
    }

    pub fn set_i2c_disable(&mut self, disable: bool) -> Result<(), E> {
        self.set_register(Register::UserControl, 2, 1, disable as u8)
    }

    /// set DLPF before set sample rate
    pub fn set_sample_rate(&mut self, rate: u16) -> Result<(), E> {
        let divider = if !self.dlpf_enabled { 8_000 } else { 1_000 } / rate - 1;
        self.bus.write(Register::SampleRateDivider, divider as u8)
    }

    pub fn set_int_pin_config(&mut self, pin_config: IntPinConfig, enable: bool) -> Result<(), E> {
        self.set_register(Register::IntPinConfig, pin_config as u8, 1, enable as u8)
    }

    pub fn set_interrupt_enable(&mut self, interrupt: Interrupt, enable: bool) -> Result<(), E> {
        self.set_register(Register::InterruptEnable, interrupt as u8, 1, enable as u8)
    }

    pub fn enable_fifo(&mut self, fifo_enable: FifoEnable) -> Result<(), E> {
        let value: u8 = fifo_enable.into();
        self.bus.write(Register::FifoEnable, value)
    }

    pub fn enable_fifo_buffer(&mut self) -> Result<(), E> {
        let value = self.bus.read(Register::UserControl)?;
        self.bus.write(Register::UserControl, value | 1 << 6)
    }

    pub fn get_fifo_counter(&mut self) -> Result<u16, E> {
        let high = self.bus.read(Register::FifoCountHigh)?;
        let low = self.bus.read(Register::FifoCountLow)?;
        return Ok((high as u16) << 8 | low as u16);
    }

    pub fn set_gyro_range(&mut self, range: GyroRange) -> Result<(), E> {
        self.bus.write(Register::GyroConfig, (range as u8) << 3)
    }

    pub fn read_acceleration(&mut self) -> Result<Acceleration, E> {
        let mut buffer = [0u8; 6];
        self.bus.reads(Register::AccelerometerXHigh, &mut buffer)?;
        Ok(buffer[..].into())
    }

    pub fn read_gyro(&mut self) -> Result<Gyro, E> {
        let mut buffer = [0u8; 6];
        self.bus.reads(Register::GyroXHigh, &mut buffer)?;
        Ok(buffer[..].into())
    }

    pub fn read_temperature(&mut self) -> Result<Temperature, E> {
        let mut buffer = [0u8; 2];
        self.bus.reads(Register::AccelerometerXHigh, &mut buffer)?;
        Ok(buffer[..].into())
    }

    pub fn read_all(&mut self) -> Result<(Acceleration, Temperature, Gyro), E> {
        let mut buffer = [0u8; 14];
        self.bus.reads(Register::AccelerometerXHigh, &mut buffer)?;
        Ok((buffer[..6].into(), buffer[6..8].into(), buffer[8..].into()))
    }

    pub fn set_accelerometer_range(&mut self, range: AccelerometerRange) -> Result<(), E> {
        self.bus.write(Register::AccelerometerConfig, (range as u8) << 3)
    }
}

impl<BUS> MPU6000<BUS> {
    pub fn free(self) -> BUS {
        self.bus
    }
}

mod test {
    use embedded_hal::blocking::delay::{DelayMs, DelayUs};
    use embedded_hal::blocking::spi::{Transfer, Write};
    use embedded_hal::digital::v2::OutputPin;

    struct StubSPI {}

    impl Write<u8> for StubSPI {
        type Error = &'static str;
        fn write(&mut self, _bytes: &[u8]) -> Result<(), &'static str> {
            Ok(())
        }
    }

    impl Transfer<u8> for StubSPI {
        type Error = &'static str;
        fn transfer<'w>(&mut self, bytes: &'w mut [u8]) -> Result<&'w [u8], &'static str> {
            for b in bytes.iter_mut() {
                *b = 100;
            }
            Ok(bytes)
        }
    }

    struct StubOutputPin {}
    impl OutputPin for StubOutputPin {
        type Error = &'static str;
        fn set_high(&mut self) -> Result<(), &'static str> {
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), &'static str> {
            Ok(())
        }
    }

    struct Nodelay {}

    impl DelayMs<u8> for Nodelay {
        fn delay_ms(&mut self, _ms: u8) {}
    }

    impl DelayUs<u8> for Nodelay {
        fn delay_us(&mut self, _us: u8) {}
    }

    #[test]
    fn test_functional() {
        extern crate std;

        use crate::bus::SpiBus;
        use crate::registers::{AccelerometerRange, GyroRange};
        use crate::MPU6000;

        let spi_bus = SpiBus::new(StubSPI {}, StubOutputPin {}, Nodelay {});
        let mut mpu6000 = MPU6000::new(spi_bus);
        let mut delay = Nodelay {};
        mpu6000.reset(&mut delay).ok();
        mpu6000.set_sleep(false).ok();
        mpu6000.set_accelerometer_range(AccelerometerRange::G16).ok();
        mpu6000.set_gyro_range(GyroRange::DPS2000).ok();
        mpu6000.read_all().ok();
    }
}
