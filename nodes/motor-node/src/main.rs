use std::path::PathBuf;

use clap::Parser;
use mechanum_protos::MotorCommand;
use pololu_motoron::ControllerType;
use robotica::{LogConfig, Node, Subscriber};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "/dev/i2c-0")]
    device: PathBuf,
    #[arg(short, long, default_value_t = 0x10)]
    address: u16,
    #[arg(short, long, default_value = "front")]
    controller_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let node = Node::new_with_logging("motor-node", LogConfig::new()).await?;
    let mut controller =
        pololu_motoron::Device::new(ControllerType::M2T256, args.device, args.address)?;
    let topic_name = format!("robot/chassis/motors/{}", args.controller_name);
    let sub: Subscriber<MotorCommand> = node.subscribe(topic_name).await?;

    while let Ok(msg) = sub.recv().await {
        let msg = msg.message;
        let motor_id = match msg.motor_id() {
            mechanum_protos::MotorId::A => 0,
            mechanum_protos::MotorId::B => 1,
        };
        controller.set_speed(motor_id, msg.speed)?;
    }
    Ok(())
}
