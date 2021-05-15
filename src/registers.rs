//! All constants used in the driver, mostly register addresses

pub enum ClockSelection {
    Internal8Mhz = 0,
    PLLWithXAxisGyroRef = 1,
    PLLWithYAxisGyroRef = 2,
    PLLWithZAxisGyroRef = 3,
    PLLWithExternal32_768KHZ = 4,
    PLLWithExternal19_2MHZ = 5,
    StopClock = 7,
}

pub enum PowerManagement1 {
    DeviceReset = 1 << 7,
}

pub enum SignalPathReset {
    GyroReset = 1 << 2,
    AccelerometerReset = 1 << 1,
    TemperatureReset = 1,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AccelerometerRange {
    /// +/- 2g, 16384 LSB/g
    G2 = 0,
    /// +/- 4g, 8192 LSB/g
    G4 = 1,
    /// +/- 8g, 4096 LSB/g
    G8 = 2,
    /// +/- 16g, 2048 LSB/g
    G16 = 3,
}

impl AccelerometerRange {
    pub fn scale_factor(&self) -> f32 {
        match self {
            AccelerometerRange::G2 => 16384.0,
            AccelerometerRange::G4 => 8192.0,
            AccelerometerRange::G8 => 4096.0,
            AccelerometerRange::G16 => 2048.0,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum GyroRange {
    /// +/- 250°/s,  131 LSB/°/s
    DPS250 = 0,
    /// +/- 500°/s, 65.5 LSB/°/s
    DPS500 = 1,
    /// +/- 1000°/s, 32.8 LSB/°/s
    DPS1000 = 2,
    /// +/- 2000°/s, 16.4 LSB/°/s
    DPS2000 = 3,
}

impl GyroRange {
    pub fn scale_factor(&self) -> f32 {
        match self {
            GyroRange::DPS250 => 131.0,
            GyroRange::DPS500 => 65.5,
            GyroRange::DPS1000 => 32.8,
            GyroRange::DPS2000 => 16.4,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum ProductId {
    Unknown,
    MPU6000ES_REV_C4,
    MPU6000ES_REV_C5,
    MPU6000ES_REV_D6,
    MPU6000ES_REV_D7,
    MPU6000ES_REV_D8,
    MPU6000_REV_C4,
    MPU6000_REV_C5,
    MPU6000_REV_D6,
    MPU6000_REV_D7,
    MPU6000_REV_D8,
    MPU6000_REV_D9,
    MPU6000_REV_D10,
}

impl From<u8> for ProductId {
    fn from(value: u8) -> Self {
        match value {
            0x14 => ProductId::MPU6000_REV_C4,
            0x15 => ProductId::MPU6000ES_REV_C5,
            0x16 => ProductId::MPU6000ES_REV_D6,
            0x17 => ProductId::MPU6000ES_REV_D7,
            0x18 => ProductId::MPU6000ES_REV_D8,
            0x54 => ProductId::MPU6000_REV_C4,
            0x55 => ProductId::MPU6000_REV_C5,
            0x56 => ProductId::MPU6000_REV_D6,
            0x57 => ProductId::MPU6000_REV_D7,
            0x58 => ProductId::MPU6000_REV_D8,
            0x59 => ProductId::MPU6000_REV_D9,
            0x5A => ProductId::MPU6000_REV_D10,
            _ => ProductId::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    ProductId = 0xc,
    SampleRateDivider = 0x19,
    Configuration = 0x1a,
    GyroConfig = 0x1b,
    AccelerometerConfig = 0x1c,
    FifoEnable = 0x23,
    IntPinConfig = 0x37,
    InterruptEnable = 0x38,
    AccelerometerXHigh = 0x3b,
    AccelerometerXLow = 0x3c,
    AccelerometerYHigh = 0x3d,
    AccelerometerYLow = 0x3e,
    AccelerometerZHigh = 0x3f,
    AccelerometerZLow = 0x40,
    TemperatureHigh = 0x41,
    TemperatureLow = 0x42,
    GyroXHigh = 0x43,
    GyroXLow = 0x44,
    GyroYHigh = 0x45,
    GyroYLow = 0x46,
    GyroZHigh = 0x47,
    GyroZLow = 0x48,
    SignalPathReset = 0x68,
    UserControl = 0x6a,
    /// Register to control chip waking from sleep, enabling sensors, default: sleep
    PowerManagement1 = 0x6b,
    /// Internal register to check slave addr
    PowerManagement2 = 0x6c,
    FifoCountHigh = 0x72,
    FifoCountLow = 0x73,
    FifoReadWrite = 0x74,
    WhoAmI = 0x75,
}
