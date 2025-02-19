use clap::{Parser, ValueEnum};
use gilrs::{
    ev::{state::GamepadState, Code},
    Axis, GamepadId, Gilrs,
};
use mechanum_protos::{DifferentialChassisCommand, MechanumChassisCommand, TankChassisCommand};
use robotica::{tracing::info, LogConfig, Node, Publisher};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum ChassisControlType {
    Tank,
    Mechanum,
    Differential,
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    control_type: ChassisControlType,

    /// ID of the gamepad. If not provided, we'll grab the first available gamepad.
    #[arg(short, long)]
    id: Option<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let node = Node::new_with_logging("gamepad-chassis", LogConfig::new()).await?;

    let gilrs = Gilrs::new().unwrap();
    let (gamepad_id, gamepad) = if let Some(id) = args.id {
        gilrs
            .gamepads()
            .find(|(gid, _)| gid.to_string() == id.to_string())
    } else {
        gilrs.gamepads().next()
    }
    .ok_or(anyhow::anyhow!("could not find a valid gamepad to use"))?;
    info!("Using gamepad {}", gamepad.name());
    let axis_map: HashMap<Axis, Code> = HashMap::from_iter(
        [
            Axis::LeftStickX,
            Axis::LeftStickY,
            Axis::RightStickX,
            Axis::RightStickY,
            Axis::DPadX,
            Axis::DPadY,
            Axis::LeftZ,
            Axis::RightZ,
        ]
        .into_iter()
        .filter_map(|a| gamepad.axis_code(a).map(|c| (a, c))),
    );
    info!("Axis map: {axis_map:?}");
    let gamepad_state: Arc<RwLock<GamepadState>> = Arc::new(RwLock::new(gamepad.state().clone()));
    let gamepad_state_clone = gamepad_state.clone();

    let gamepad_jh =
        tokio::task::spawn_blocking(move || gamepad_update_loop(gamepad_state, gilrs, gamepad_id));

    tokio::select! {
        r = gamepad_jh => r?,
        r = chassis_loop_match(&node, args, gamepad_state_clone, axis_map) => r,
    }
}

fn gamepad_update_loop(
    gamepad_state: Arc<RwLock<GamepadState>>,
    mut gilrs: Gilrs,
    gamepad_id: GamepadId,
) -> anyhow::Result<()> {
    loop {
        // Accumulate all changes that happened
        while gilrs.next_event().is_some() {}
        // Update after
        *gamepad_state.blocking_write() = gilrs.gamepad(gamepad_id).state().clone();
    }
}

async fn chassis_loop_match(
    node: &Node,
    args: Args,
    gamepad_state: Arc<RwLock<GamepadState>>,
    axis_map: HashMap<Axis, Code>,
) -> anyhow::Result<()> {
    match args.control_type {
        ChassisControlType::Tank => {
            chassis_control_loop::<TankControl>(&node, gamepad_state, axis_map).await
        }
        ChassisControlType::Mechanum => {
            chassis_control_loop::<MechanumControl>(&node, gamepad_state, axis_map).await
        }
        ChassisControlType::Differential => {
            chassis_control_loop::<DifferentialControl>(node, gamepad_state, axis_map).await
        }
    }
}

async fn chassis_control_loop<C: ChassisControl>(
    node: &Node,
    gamepad_state: Arc<RwLock<GamepadState>>,
    axis_map: HashMap<Axis, Code>,
) -> anyhow::Result<()> {
    let publisher: Publisher<C::OutputMessage> = node.publish(C::TOPIC_NAME).await?;
    loop {
        let msg = {
            let gamepad_state = gamepad_state.read().await;
            C::generate_command(&gamepad_state, &axis_map)
        };
        publisher.send(&msg).await?;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

trait ChassisControl {
    type OutputMessage: prost::Name + std::fmt::Debug;
    const TOPIC_NAME: &'static str;
    fn generate_command(
        gamepad_state: &GamepadState,
        axis_map: &HashMap<Axis, Code>,
    ) -> Self::OutputMessage;
}

struct TankControl;
impl ChassisControl for TankControl {
    type OutputMessage = TankChassisCommand;
    const TOPIC_NAME: &'static str = "robot/chassis/tank";
    fn generate_command(
        gamepad_state: &GamepadState,
        axis_map: &HashMap<Axis, Code>,
    ) -> TankChassisCommand {
        let right = get_axis_value(gamepad_state, Axis::RightStickY, axis_map).clamp(-1., 1.);
        let left = get_axis_value(gamepad_state, Axis::LeftStickY, axis_map).clamp(-1., 1.);

        TankChassisCommand { left, right }
    }
}

struct DifferentialControl;
impl ChassisControl for DifferentialControl {
    type OutputMessage = DifferentialChassisCommand;
    const TOPIC_NAME: &'static str = "robot/chassis/simple";
    fn generate_command(
        gamepad_state: &GamepadState,
        axis_map: &HashMap<Axis, Code>,
    ) -> DifferentialChassisCommand {
        let speed = get_axis_value(gamepad_state, Axis::LeftStickY, axis_map).clamp(-1., 1.);
        let rotation = get_axis_value(gamepad_state, Axis::LeftStickX, axis_map).clamp(-1., 1.);

        DifferentialChassisCommand { speed, rotation }
    }
}

struct MechanumControl;
impl ChassisControl for MechanumControl {
    type OutputMessage = MechanumChassisCommand;
    const TOPIC_NAME: &'static str = "robot/chassis/mechanum";
    fn generate_command(
        gamepad_state: &GamepadState,
        axis_map: &HashMap<Axis, Code>,
    ) -> MechanumChassisCommand {
        let longitudinal = get_axis_value(gamepad_state, Axis::LeftStickY, axis_map).clamp(-1., 1.);
        let lateral = get_axis_value(gamepad_state, Axis::LeftStickX, axis_map).clamp(-1., 1.);
        let rotation = get_axis_value(gamepad_state, Axis::RightStickX, axis_map).clamp(-1., 1.);

        MechanumChassisCommand {
            longitudinal,
            lateral,
            rotation,
        }
    }
}

fn get_axis_value(gamepad_state: &GamepadState, axis: Axis, axis_map: &HashMap<Axis, Code>) -> f32 {
    gamepad_state
        .axis_data(*axis_map.get(&axis).expect("no code for axis"))
        .map(|d| d.value())
        .unwrap_or(0.0)
}
