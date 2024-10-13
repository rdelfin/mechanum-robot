use mechanum_protos::{Direction, MotorCommand, MotorId};
use robotica::{LogConfig, Node, Publisher};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("mechanum-chassis-node", LogConfig::new()).await?;
    let publisher: Publisher<MotorCommand> = node.publish("robot/chassis/motors").await?;

    loop {
        publisher
            .send(&motor_command(MotorId::FrontLeft, Direction::Forward, 255))
            .await?;
        publisher
            .send(&motor_command(MotorId::FrontRight, Direction::Forward, 255))
            .await?;
        publisher
            .send(&motor_command(MotorId::BackLeft, Direction::Forward, 255))
            .await?;
        publisher
            .send(&motor_command(MotorId::BackRight, Direction::Forward, 255))
            .await?;

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {},
            _ = tokio::signal::ctrl_c() => { break; },
        }
    }

    publisher
        .send(&motor_command(MotorId::FrontLeft, Direction::Forward, 0))
        .await?;
    publisher
        .send(&motor_command(MotorId::FrontRight, Direction::Forward, 0))
        .await?;
    publisher
        .send(&motor_command(MotorId::BackLeft, Direction::Forward, 0))
        .await?;
    publisher
        .send(&motor_command(MotorId::BackRight, Direction::Forward, 0))
        .await?;
    Ok(())
}

fn motor_command(motor_id: MotorId, direction: Direction, pwm_val: i32) -> MotorCommand {
    MotorCommand {
        motor_id: motor_id.into(),
        direction: direction.into(),
        pwm_val,
    }
}
