use mechanum_protos::{Direction, MotorCommand, MotorId, TankChassisCommand};
use robotica::{LogConfig, Node, Publisher, Subscriber};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("mechanum-chassis-node", LogConfig::new()).await?;
    let subscriber: Subscriber<TankChassisCommand> = node.subscribe("robot/chassis/tank").await?;
    let publisher: Publisher<MotorCommand> = node.publish("robot/chassis/motors").await?;
    let chassis_cmd = Arc::new(RwLock::new(TankChassisCommand::default()));

    tokio::select! {
        r = recv_loop(subscriber, chassis_cmd.clone()) => { r },
        r = publish_loop(publisher, chassis_cmd, Duration::from_millis(100)) => { r },
        _ = tokio::signal::ctrl_c() => { return Ok(()) },
    }?;

    Ok(())
}

async fn recv_loop(
    subscriber: Subscriber<'_, TankChassisCommand>,
    chassis_cmd: Arc<RwLock<TankChassisCommand>>,
) -> anyhow::Result<()> {
    loop {
        let msg = subscriber.recv().await?;
        *chassis_cmd.write().await = msg.message;
    }
}

async fn publish_loop(
    publisher: Publisher<'_, MotorCommand>,
    chassis_cmd: Arc<RwLock<TankChassisCommand>>,
    period: Duration,
) -> anyhow::Result<()> {
    loop {
        let current_cmd = chassis_cmd.read().await.clone();

        publisher
            .send(&motor_command(MotorId::FrontLeft, current_cmd.left))
            .await?;
        publisher
            .send(&motor_command(MotorId::FrontRight, current_cmd.right))
            .await?;
        publisher
            .send(&motor_command(MotorId::BackLeft, current_cmd.left))
            .await?;
        publisher
            .send(&motor_command(MotorId::BackRight, current_cmd.right))
            .await?;

        tokio::time::sleep(period).await;
    }
}

fn motor_command(motor_id: MotorId, val: f32) -> MotorCommand {
    MotorCommand {
        motor_id: motor_id.into(),
        direction: if val >= 0. {
            Direction::Forward
        } else {
            Direction::Backward
        }
        .into(),
        pwm_val: (val.abs().min(1.) * 255.).round() as i32,
    }
}
