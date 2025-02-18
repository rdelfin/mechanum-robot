use mechanum_protos::{MotorCommand, MotorId, TankChassisCommand};
use robotica::{LogConfig, Node, Publisher, Subscriber};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

struct RawChassisCommand {
    front_left: f32,
    front_right: f32,
    back_left: f32,
    back_right: f32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("mechanum-chassis-node", LogConfig::new()).await?;
    let publisher_front: Publisher<MotorCommand> =
        node.publish("robot/chassis/motors/front").await?;
    let publisher_back: Publisher<MotorCommand> = node.publish("robot/chassis/motors/back").await?;

    run_with_control::<TankControls>(&node, publisher_front, publisher_back).await
}

async fn run_with_control<'n, C: MotorControl>(
    node: &'n Node,
    publisher_front: Publisher<'n, MotorCommand>,
    publisher_back: Publisher<'n, MotorCommand>,
) -> anyhow::Result<()> {
    let subscriber: Subscriber<C::Command> = node.subscribe(C::TOPIC_NAME).await?;
    let chassis_cmd = Arc::new(RwLock::new(C::Command::default()));

    tokio::select! {
        r = recv_loop(subscriber, chassis_cmd.clone()) => { r },
        r = publish_loop::<C>(publisher_front, publisher_back, chassis_cmd, Duration::from_millis(100)) => { r },
        _ = tokio::signal::ctrl_c() => { return Ok(()) },
    }?;
    Ok(())
}

async fn recv_loop<C: Default + prost::Name>(
    subscriber: Subscriber<'_, C>,
    chassis_cmd: Arc<RwLock<C>>,
) -> anyhow::Result<()> {
    loop {
        let msg = subscriber.recv().await?;
        *chassis_cmd.write().await = msg.message;
    }
}

async fn publish_loop<C: MotorControl>(
    publisher_front: Publisher<'_, MotorCommand>,
    publisher_back: Publisher<'_, MotorCommand>,
    chassis_cmd: Arc<RwLock<C::Command>>,
    period: Duration,
) -> anyhow::Result<()> {
    loop {
        let command = {
            let current_cmd = chassis_cmd.read().await;
            C::controls(&current_cmd)
        };

        publisher_front
            .send(&motor_command(MotorId::B, command.front_left))
            .await?;
        publisher_front
            .send(&motor_command(MotorId::A, command.front_right))
            .await?;
        publisher_back
            .send(&motor_command(MotorId::B, command.back_left))
            .await?;
        publisher_back
            .send(&motor_command(MotorId::A, command.back_right))
            .await?;

        tokio::time::sleep(period).await;
    }
}

trait MotorControl {
    const TOPIC_NAME: &'static str;
    type Command: Default + prost::Name;
    fn controls(cmd: &Self::Command) -> RawChassisCommand;
}

struct TankControls;
impl MotorControl for TankControls {
    const TOPIC_NAME: &'static str = "robot/chassis/tank";
    type Command = TankChassisCommand;
    fn controls(msg: &TankChassisCommand) -> RawChassisCommand {
        RawChassisCommand {
            front_left: msg.left,
            front_right: msg.right,
            back_left: msg.left,
            back_right: msg.right,
        }
    }
}

fn motor_command(motor_id: MotorId, speed: f32) -> MotorCommand {
    MotorCommand {
        motor_id: motor_id.into(),
        speed,
    }
}
