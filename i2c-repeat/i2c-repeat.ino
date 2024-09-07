#include <Wire.h>
#define SLAVE_ADDRESS (0x65)

byte data_to_echo = 0;

void setup() {
    Wire.begin(SLAVE_ADDRESS);
    Serial.begin(9600);
    Serial.println("------------------------------ I am slave 0x65");
    delay(1000);
    Wire.onReceive(receiveData);
    Wire.onRequest(sendData);
}

void loop() { }

void receiveData(int bytecount) {
    Serial.print("Received ");
    Serial.print(bytecount);
    Serial.println(" byte(s)");
    for (int i = 0; i < bytecount; i++) {
        data_to_echo = Wire.read();
        Serial.print("Got byte ");
        Serial.println(data_to_echo);
    }
}

void sendData() {
    Serial.println("Got a request to echo data");
    Wire.write(data_to_echo);
}
