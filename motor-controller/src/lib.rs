use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use int_enum::IntEnum;
use std::path::Path;

pub const MOTOR_CONTROLLER_ADDRESS: u16 = 0x65;

/// This is a set of all errors returned by this library.
#[derive(thiserror::Error, Debug)]
pub enum MotorError {
    /// This is returned any time we have an error communicating with the i2c bus. This can mean
    /// that there is not i2c slave at the address, or the device doesn't exist, or there was an
    /// error returned by the relevant syscalls. All these and more fit in this case
    #[error("got error from I2C driver: {0}")]
    I2cError(#[from] i2cdev::linux::LinuxI2CError),
    /// This is returned from [`MotorController::send_command`] if we get an error code back from
    /// the motor controller. All possible errors returned should be accounted by this library
    /// however, so if you get this error, it is likely a bug.
    #[error("unknown error from motor controller with code, likely a bug: {0}")]
    UnknownMotorError(u8),
}

/// The direction in which a wheel is moving. While the position is entirely arbitrary and
/// dependant on how you've wired each motor to the arduino, we do recommend making sure all motors
/// will spin in the same direction if given the same direction command, and to have the `Forward`
/// direction match with the side the front motors are on.
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

/// An identifier for each of the 4 motors that the motor controller can address. While the naming
/// and exact position is arbitrary and dependant on how you've wired the motors to the arduino, we
/// do recommend wiring in such a way that there is an orientation of the robot where this order is
/// accurate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntEnum)]
#[repr(u8)]
pub enum Motor {
    FrontLeft = 0,
    FrontRight = 1,
    BackLeft = 2,
    BackRight = 3,
}

/// This struct lets you control a motor controller as defined in the arduino sketches of this
/// project. It communicates over I2C by sending individual motor commands and checking for a
/// status code response.
pub struct MotorController {
    dev: LinuxI2CDevice,
}

impl MotorController {
    /// Creates a new motor controller with the default i2c address, but an explicit device path as
    /// this is very device-specific.
    pub fn new<P: AsRef<Path>>(dev: P) -> Result<MotorController, MotorError> {
        Self::with_i2c_address(dev, MOTOR_CONTROLLER_ADDRESS)
    }

    /// Creates a new motor controller with explicit i2c address and device, if you've configured
    /// things differently from what the defaults are.
    pub fn with_i2c_address<P: AsRef<Path>>(
        dev: P,
        address: u16,
    ) -> Result<MotorController, MotorError> {
        let dev = LinuxI2CDevice::new(dev, address)?;
        Ok(MotorController { dev })
    }

    /// This function will send a command down to the motor controller. We strive to avoid errors
    /// by making the parameters fool-proof: all combinations of arguments are valid. As such, the
    /// only errors this function should return are relating to the i2c bus setup, permissions, and
    /// other things outside the scope of the protocol itself.
    ///
    /// # Arguments
    /// * `motor_id`  - Which motor to change the settings for. Positioning is arbitrary and
    ///                 dependant on wiring.
    /// * `pwm_val`   - What speed to enable the motor at, where 0 is off and 255 is full voltage.
    /// * `direction` - Which direction to move the motor in. This direction is arbitrary and
    ///                 depends on the exact wiring, but we interpret one as fwd and one as bwd.
    pub fn send_command(
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
