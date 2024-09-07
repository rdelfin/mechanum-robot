from smbus2 import SMBus, i2c_msg
import time

class MotorArduinoI2C:
    def __init__(self, bus: SMBus, address: int):
        self.bus = bus
        self.address = address

    def set_motor(self, pwm_val: int, direction: bool, motor_id: int):
        if pwm_val < 0 or pwm_val > 255:
            raise ValueError("Invalid PWM val, should be between 0 and 255")
        if motor_id > 3 or motor_id < 0:
            raise ValueError("Invalid motor ID, should be between 0 and 3")
        dir_char = "+"
        if not direction:
            dir_char = "-"

        pwm_val_str = str(pwm_val).zfill(3)
        command = f"{motor_id}{dir_char}{pwm_val_str}".encode()
        print(f"Sending command: {command}")
        self.bus.write_block_data(self.address, 0, command)
        response = bus.read_byte(self.address)
        if response != 0:
            raise RuntimeError(f'error from i2c bus: "{response}"')

bus = SMBus(1)
address = 0x65
mtr = MotorArduinoI2C(bus, address)
time.sleep(1)

mtr.set_motor(255, True, 0)
print("Set motor 0 to 255")
time.sleep(1)

mtr.set_motor(100, False, 1)
print("Set motor 1 to -100")
time.sleep(1)

mtr.set_motor(200, True, 2)
print("Set motor 2 to 200")
time.sleep(1)

mtr.set_motor(230, False, 3)
print("Set motor 3 to -230")
time.sleep(1)

mtr.set_motor(255, False, 0)
mtr.set_motor(255, False, 1)
mtr.set_motor(255, False, 2)
mtr.set_motor(255, False, 3)
print("Set all motors to full speed")
time.sleep(1)

mtr.set_motor(0, True, 0)
mtr.set_motor(0, True, 1)
mtr.set_motor(0, True, 2)
mtr.set_motor(0, True, 3)
print("Reset all motors to 0")
time.sleep(1)
