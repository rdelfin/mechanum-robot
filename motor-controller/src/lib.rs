use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use int_enum::IntEnum;
use std::path::Path;

const MOTOR_CONTROLLER_ADDRESS: u16 = 0x52;

#[derive(thiserror::Error, Debug)]
pub enum MotorError {
    #[error("got error from I2C driver: {0}")]
    I2cError(#[from] i2cdev::linux::LinuxI2CError),
    #[error("unknown error from motor controller with code, likely a bug: {0}")]
    UnknownMotorError(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

impl From<Direction> for &'static str {
    fn from(d: Direction) -> &'static str {
        match d {
            Direction::Forward => "+",
            Direction::Backward => "-",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntEnum)]
#[repr(u8)]
pub enum Motor {
    FrontLeft = 0,
    FrontRight = 1,
    BackLeft = 2,
    BackRight = 3,
}

pub struct MotorController {
    dev: LinuxI2CDevice,
}

impl MotorController {
    pub fn new<P: AsRef<Path>>(dev: P) -> Result<MotorController, MotorError> {
        Self::with_i2c_address(dev, MOTOR_CONTROLLER_ADDRESS)
    }

    pub fn with_i2c_address<P: AsRef<Path>>(
        dev: P,
        address: u16,
    ) -> Result<MotorController, MotorError> {
        let dev = LinuxI2CDevice::new(dev, address)?;
        Ok(MotorController { dev })
    }

    pub fn send_request(
        &mut self,
        motor_id: Motor,
        pwm_val: u8,
        direction: Direction,
    ) -> Result<(), MotorError> {
        // Write the motor intent first
        let dir_str: &'static str = direction.into();
        let msg = format!("{}{}{:0>3}", u8::from(motor_id), dir_str, pwm_val);
        self.dev.smbus_write_block_data(0, msg.as_bytes())?;

        // Read back the result
        let err_code = self.dev.smbus_read_byte()?;
        if err_code == 0 {
            Ok(())
        } else {
            Err(MotorError::UnknownMotorError(err_code))
        }
    }
}
