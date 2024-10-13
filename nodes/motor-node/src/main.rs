use mechanum_protos::MotorCommand;
use motor_controller::{Direction, Motor, MotorController};
use robotica::{LogConfig, Node, Subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("motor-node", LogConfig::new()).await?;
    let mut controller = MotorController::new("/dev/i2c-1")?;
    let sub: Subscriber<MotorCommand> = node.subscribe("robot/chassis/motors").await?;

    while let Ok(msg) = sub.recv().await {
        let msg = msg.message;
        let motor_id = match msg.motor_id() {
            mechanum_protos::MotorId::FrontLeft => Motor::FrontLeft,
            mechanum_protos::MotorId::FrontRight => Motor::FrontRight,
            mechanum_protos::MotorId::BackLeft => Motor::BackLeft,
            mechanum_protos::MotorId::BackRight => Motor::BackRight,
        };
        let direction = match msg.direction() {
            mechanum_protos::Direction::Forward => Direction::Forward,
            mechanum_protos::Direction::Backward => Direction::Backward,
        };
        let pwm_val: u8 = msg
            .pwm_val
            .clamp(0, 255)
            .try_into()
            .expect("value must be in u8 range");
        controller.send_command(motor_id, pwm_val, direction)?;
    }
    Ok(())
}
