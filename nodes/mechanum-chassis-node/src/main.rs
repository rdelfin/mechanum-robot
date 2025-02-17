use mechanum_protos::{MotorCommand, MotorId, TankChassisCommand};
use robotica::{LogConfig, Node, Publisher, Subscriber};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("mechanum-chassis-node", LogConfig::new()).await?;
    let subscriber: Subscriber<TankChassisCommand> = node.subscribe("robot/chassis/tank").await?;
    let publisher_front: Publisher<MotorCommand> =
        node.publish("robot/chassis/motors/front").await?;
    let publisher_back: Publisher<MotorCommand> = node.publish("robot/chassis/motors/back").await?;
    let chassis_cmd = Arc::new(RwLock::new(TankChassisCommand::default()));

    tokio::select! {
        r = recv_loop(subscriber, chassis_cmd.clone()) => { r },
        r = publish_loop(publisher_front, publisher_back, chassis_cmd, Duration::from_millis(100)) => { r },
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
    publisher_front: Publisher<'_, MotorCommand>,
    publisher_back: Publisher<'_, MotorCommand>,
    chassis_cmd: Arc<RwLock<TankChassisCommand>>,
    period: Duration,
) -> anyhow::Result<()> {
    loop {
        let current_cmd = chassis_cmd.read().await.clone();

        publisher_front
            .send(&motor_command(MotorId::B, current_cmd.left))
            .await?;
        publisher_front
            .send(&motor_command(MotorId::A, current_cmd.right))
            .await?;
        publisher_back
            .send(&motor_command(MotorId::B, current_cmd.left))
            .await?;
        publisher_back
            .send(&motor_command(MotorId::A, current_cmd.right))
            .await?;

        tokio::time::sleep(period).await;
    }
}

fn motor_command(motor_id: MotorId, speed: f32) -> MotorCommand {
    MotorCommand {
        motor_id: motor_id.into(),
        speed,
    }
}
