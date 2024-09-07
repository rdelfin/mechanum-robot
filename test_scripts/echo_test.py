from smbus2 import SMBus
import time

def send_msg(bus, address, c: str):
    bus.write_block_data(address, 0, c.encode())
    response = chr(bus.read_byte(address))
    print(f"Got response '{response}' (aka {ord(response):#x})")
    return response

bus = SMBus(1)
address = 0x65
time.sleep(1)


send_msg(bus, address, "0+1")
time.sleep(1)
send_msg(bus, address, "0+100")
time.sleep(1)
send_msg(bus, address, "0+1")
time.sleep(1)
send_msg(bus, address, "0-100")
time.sleep(1)
send_msg(bus, address, "0+255")
time.sleep(1)
send_msg(bus, address, "0+000")
time.sleep(1)
