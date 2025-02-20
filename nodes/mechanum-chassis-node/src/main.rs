use clap::{Parser, ValueEnum};
use mechanum_protos::{
    DifferentialChassisCommand, MechanumChassisCommand, MotorCommand, MotorId, TankChassisCommand,
};
use robotica::{LogConfig, Node, Publisher, Subscriber};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 't', long)]
    control_type: MotorControlType,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let node = Node::new_with_logging("mechanum-chassis-node", LogConfig::new()).await?;
    let publisher_front: Publisher<MotorCommand> =
        node.publish("robot/chassis/motors/front").await?;
    let publisher_back: Publisher<MotorCommand> = node.publish("robot/chassis/motors/back").await?;

    match args.control_type {
        MotorControlType::Tank => {
            run_with_control::<TankControls>(&node, publisher_front, publisher_back).await
        }
        MotorControlType::Mechanum => {
            run_with_control::<MechanumControls>(&node, publisher_front, publisher_back).await
        }
        MotorControlType::Differential => {
            run_with_control::<DifferentialControls>(&node, publisher_front, publisher_back).await
        }
    }
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

#[derive(Debug, Clone, Default, PartialEq)]
struct RawChassisCommand {
    front_left: f32,
    front_right: f32,
    back_left: f32,
    back_right: f32,
}

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum MotorControlType {
    Tank,
    Mechanum,
    Differential,
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

struct DifferentialControls;
impl MotorControl for DifferentialControls {
    const TOPIC_NAME: &'static str = "robot/chassis/simple";
    type Command = DifferentialChassisCommand;
    fn controls(msg: &DifferentialChassisCommand) -> RawChassisCommand {
        let right = clamp_with_nan(msg.speed - msg.rotation);
        let left = clamp_with_nan(msg.speed + msg.rotation);
        RawChassisCommand {
            front_left: left,
            front_right: right,
            back_left: left,
            back_right: right,
        }
    }
}

struct MechanumControls;
impl MotorControl for MechanumControls {
    const TOPIC_NAME: &'static str = "robot/chassis/mechanum";
    type Command = MechanumChassisCommand;
    fn controls(msg: &MechanumChassisCommand) -> RawChassisCommand {
        RawChassisCommand {
            front_left: clamp_with_nan((msg.longitudinal + msg.lateral + msg.rotation) / 2.),
            front_right: clamp_with_nan((msg.longitudinal - msg.lateral - msg.rotation) / 2.),
            back_left: clamp_with_nan((msg.longitudinal - msg.lateral + msg.rotation) / 2.),
            back_right: clamp_with_nan((msg.longitudinal + msg.lateral - msg.rotation) / 2.),
        }
    }
}

fn motor_command(motor_id: MotorId, speed: f32) -> MotorCommand {
    MotorCommand {
        motor_id: motor_id.into(),
        speed,
    }
}

fn clamp_with_nan(v: f32) -> f32 {
    if v.is_nan() {
        0.
    } else {
        v.clamp(-1., 1.)
    }
}
