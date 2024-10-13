use mechanum_protos::MotorCommand;
use motor_controller::MotorController;
use robotica::{Node, Subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new("motor-node").await?;
    let controller = MotorController::new("/dev/i2c-1")?;
    let sub: Subscriber<MotorCommand> = node.subscribe("robot/chassis/motors").await?;

    while let Ok(msg) = sub.recv().await {
        println!("Got: {:?}", msg.message);
    }
    Ok(())
}
