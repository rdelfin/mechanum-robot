use clap::Parser;
use log::{error, info};
use mechanum_protos::MotorCommand;
use pololu_motoron::ControllerType;
use robotica::{LogConfig, Node, Subscriber};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

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
    let number_motors = ControllerType::M2T256.motor_channels();
    let mut controller =
        pololu_motoron::Device::new(ControllerType::M2T256, args.device, args.address)?;
    let topic_name = format!("robot/chassis/motors/{}", args.controller_name);
    let sub: Subscriber<MotorCommand> = node.subscribe(topic_name).await?;

    // Setup the speed data we will communicate across threads
    let initial_speed_data = vec![0.0f32; usize::from(number_motors)];
    let speed_data = Arc::new(Mutex::new(initial_speed_data));

    info!(
        "Connected to controller with version: {:?}",
        controller.firmware_version()
    );

    // Setup the writer thread
    let speed_data_clone = speed_data.clone();
    let jh = tokio::task::spawn(async move {
        if let Err(e) = write_speed(speed_data_clone, controller).await {
            error!("Error in write speed thread, exiting. Error: {e}");
            std::process::exit(-1);
        }
    });

    tokio::select! {
        res = recv_messages(sub, speed_data) => { res },
        res = jh => {
            res?;
            Err(anyhow::anyhow!("write speed thread exited unexpectedly, quitting"))
        }
    }
}

async fn recv_messages(
    sub: Subscriber<'_, MotorCommand>,
    speed_data: Arc<Mutex<Vec<f32>>>,
) -> anyhow::Result<()> {
    while let Ok(msg) = sub.recv().await {
        let msg = msg.message;
        let motor_id = match msg.motor_id() {
            mechanum_protos::MotorId::A => 0,
            mechanum_protos::MotorId::B => 1,
        };
        let mut speed_data = speed_data.lock().await;
        speed_data[motor_id] = msg.speed;
    }
    Ok(())
}

async fn write_speed(
    speed_data: Arc<Mutex<Vec<f32>>>,
    mut controller: pololu_motoron::Device,
) -> anyhow::Result<()> {
    loop {
        let speed_data = speed_data.lock().await.clone();
        controller.set_all_speeds(&speed_data)?;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
