#include <Wire.h>

// Motor 1
#define pwm1 (5)
#define pwr1Fwd (2)
#define pwr1Bwd (3)

// Motor 2
#define pwm2 (6)
#define pwr2Fwd (4)
#define pwr2Bwd (7)

// Motor 3
#define pwm3 (10)
#define pwr3Fwd (13)
#define pwr3Bwd (12)

// Motor 4
#define pwm4 (9)
#define pwr4Fwd (11)
#define pwr4Bwd (8)

// I2C
#define slaveAddress (0x65)

// Error codes
#define ERROR_WRONG_LENGTH ("01")
#define ERROR_PARSE ("02")
#define ERROR_INVALID_MOTOR ("03")
#define ERROR_INVALID_PWM ("04")

#define RETURN_ERROR(ERR_CODE) \
    Serial.print("Error: ");   \
    Serial.println(ERR_CODE);  \
    return

struct MotorData {
    bool direction = false;
    int pwmOutput = 0;
};

struct MotorData motors[4];

bool rotDirection = false;
int wheel = 0;
bool increasing = true;
int pwmOutput = 0;

void setupWheel(int pwmPin, int fwdPin, int bwdPin) {
    pinMode(pwmPin, OUTPUT);
    pinMode(fwdPin, OUTPUT);
    pinMode(bwdPin, OUTPUT);
    digitalWrite(fwdPin, HIGH);
    digitalWrite(bwdPin, HIGH);
}

void setup() {
    Wire.begin(slaveAddress);
    Serial.begin(9600);
    Serial.println("------------------------------ I am slave 0x65");
    delay(1000);
    Wire.onRequest(requestEvent);
    Wire.onReceive(receiveData);

    setupWheel(pwm1, pwr1Fwd, pwr1Bwd);
    setupWheel(pwm2, pwr2Fwd, pwr2Bwd);
    setupWheel(pwm3, pwr3Fwd, pwr3Bwd);
    setupWheel(pwm4, pwr4Fwd, pwr4Bwd);
}

void setWheelMovement(int pwm, bool direction, int pwmPin, int fwdPin, int bwdPin) {
    analogWrite(pwmPin, pwm);
    if (direction) {
        digitalWrite(fwdPin, HIGH);
        digitalWrite(bwdPin, LOW);
    } else {
        digitalWrite(fwdPin, LOW);
        digitalWrite(bwdPin, HIGH);
    }
}

bool isDigit(char c) {
    return c >= '0' && c <= '9';
}

int digitToInt(char c) {
    return int(c - '0');
}


void requestEvent() {
    Serial.println("Got request event");
    Serial.println("Sending back OK;00");
    Wire.write("OK;00");
}

// Protocol:
// We expect to receive a:
// motor number (0-3) as ASCII, a '+' or a '-' depending on the direction you
// want the motor to move in, and a 3-digit number between 000-255 (as ASCII),
// representing the PWM duty cycle. The message should therefore have exactly 5
// bytes.
// We always return 5 bytes: a status, a ';', and a 2 digit error code.
// Basically, it's always either "OK;00" or "ER;XX" where "XX" are two digits
// representing an error code.
void receiveData(int byte_count) {
    Serial.print("Got ");
    Serial.print(byte_count);
    Serial.println(" byte(s)");

    n = Wire.read();

    // First byte is always 0 for some reason, so we actually see 6
    if(byte_count != 6) {
        Serial.print("Wrong number of available chars: ");
        Serial.println(byte_count);
        RETURN_ERROR(ERROR_WRONG_LENGTH);
    }
    char first_char = Wire.read();
    if (first_char != 0) {
        Serial.print("Expected first char to be 0, saw ");
        Serial.println(int(first_char));
        RETURN_ERROR(ERROR_PARSE);
    }

    char motor_char = Wire.read();
    if (!isDigit(motor_char)) {
        Serial.print("Motor number is not a digit, is: ");
        Serial.println(int(motor_char));
        RETURN_ERROR(ERROR_PARSE);
    }
    int motor_number = digitToInt(motor_char);
    if (motor_number >= 4) {
        Serial.println("Motor number is more than 3");
        RETURN_ERROR(ERROR_INVALID_MOTOR);
    }

    char direction_char = Wire.read();
    if (direction_char != '+' && direction_char != '-') {
        Serial.println("Direction char is not valid (not + or -)");
        RETURN_ERROR(ERROR_PARSE);
    }
    bool direction = direction_char == '+' ? true : false;

    char digit1 = Wire.read();
    char digit2 = Wire.read();
    char digit3 = Wire.read();
    if (!isDigit(digit1) && !isDigit(digit2) && !isDigit(digit3)) {
        Serial.println("PWM value is not made up of 3 digits");
        RETURN_ERROR(ERROR_PARSE);
    }
    int pwm_val = digitToInt(digit1) * 100 + digitToInt(digit2) * 10 + digitToInt(digit3);
    if (pwm_val > 255) {
        Serial.println("PWM value is more than 255");
        RETURN_ERROR(ERROR_INVALID_PWM);
    }

    motors[motor_number].direction = direction;
    motors[motor_number].pwmOutput = pwm_val;
}

void loop() {
    if (increasing) {
        if (pwmOutput >= 255) {
            increasing = false;
        } else {
            pwmOutput = min(pwmOutput + 8, 255);
        }
    } else {
        if (pwmOutput <= 0) {
            if (rotDirection) wheel = (wheel + 1) % 4;
            increasing = true;
            rotDirection = !rotDirection;
        } else {
            pwmOutput = max(pwmOutput - 8, 0);
        }
    }

    setWheelMovement(motors[0].pwmOutput, motors[0].direction, pwm1, pwr1Fwd, pwr1Bwd);
    setWheelMovement(motors[1].pwmOutput, motors[1].direction, pwm2, pwr2Fwd, pwr2Bwd);
    setWheelMovement(motors[2].pwmOutput, motors[2].direction, pwm3, pwr3Fwd, pwr3Bwd);
    setWheelMovement(motors[3].pwmOutput, motors[3].direction, pwm4, pwr4Fwd, pwr4Bwd);

    delay(10);
}
