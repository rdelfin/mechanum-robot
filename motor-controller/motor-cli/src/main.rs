use clap::{Args, Parser, Subcommand};
use motor_controller::{Direction, MotorController};
use std::path::PathBuf;

/// A CLI for interacting with the Mechanum wheel motor controller.
#[derive(Parser, Debug, Clone)]
struct Cli {
    /// The path to the Linux device for this i2c bus. Usually something like `/dev/i2c-*`
    #[arg(short = 'd', long, default_value = "/dev/i2c-1")]
    i2c_device: PathBuf,
    /// The i2c slave address where we can find the motor controller.
    #[arg(short = 'a', long, default_value_t = motor_controller::MOTOR_CONTROLLER_ADDRESS)]
    i2c_address: u16,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Sends a command to a specific motor, which will set it indefinitely
    Set(SetCommand),
}

#[derive(Args, Debug, Clone)]
struct SetCommand {
    /// ID of the motor to use, from 0 to 3
    motor_id: u8,
    /// The PWM value to set, from 0 to 255
    value: u8,
    /// If this flag is set, we will flip the direction
    #[arg(short, long)]
    backward: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let controller = MotorController::with_i2c_address(args.i2c_device, args.i2c_address)?;

    match args.command {
        Command::Set(cmd) => set_cmd(controller, cmd),
    }
}

fn set_cmd(mut controller: MotorController, cmd: SetCommand) -> anyhow::Result<()> {
    let direction = if cmd.backward {
        Direction::Backward
    } else {
        Direction::Forward
    };
    let motor_id = cmd
        .motor_id
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid motor ID"))?;
    controller.send_command(motor_id, cmd.value, direction)?;
    Ok(())
}
