#include <Wire.h>

// Motor 1
#define pwm1 (5)
#define pwr1Fwd (3)
#define pwr1Bwd (2)

// Motor 2
#define pwm2 (6)
#define pwr2Fwd (7)
#define pwr2Bwd (4)

// Motor 3
#define pwm3 (10)
#define pwr3Fwd (12)
#define pwr3Bwd (13)

// Motor 4
#define pwm4 (9)
#define pwr4Fwd (8)
#define pwr4Bwd (11)

// I2C
#define slaveAddress (0x65)

// Error codes
#define ERROR_OK (0)
#define ERROR_WRONG_LENGTH (1)
#define ERROR_PARSE (2)
#define ERROR_INVALID_MOTOR (3)
#define ERROR_INVALID_PWM (4)

#define RETURN_ERROR(ERR_CODE) \
    error_code = ERR_CODE;      \
    Serial.print("Error: ");   \
    Serial.println(ERR_CODE);  \
    return

struct MotorData {
    bool direction = false;
    int pwmOutput = 0;
};

struct MotorData motors[4];

// The error code returned via i2c
char error_code = 0;

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
    Serial.print("Got request event, sending back code ");
    Serial.println(error_code);
    Wire.write(error_code);
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
    // Reset error code
    error_code = 0;

    // Buffer for our data
    char byte_data[7];

    // ALWAYS read first so we can read more data later
    for (int i = 0; i < byte_count; i++) {
        byte_data[min(i, 6)] = Wire.read();
    }

    Serial.print("Got ");
    Serial.print(byte_count);
    Serial.println(" byte(s)");

    // First two chars can be ignored
    if(byte_count != 7) {
        Serial.print("Wrong number of available chars: ");
        Serial.println(byte_count);
        RETURN_ERROR(ERROR_WRONG_LENGTH);
    }

    char motor_char = byte_data[2];
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

    char direction_char = byte_data[3];
    if (direction_char != '+' && direction_char != '-') {
        Serial.println("Direction char is not valid (not + or -)");
        RETURN_ERROR(ERROR_PARSE);
    }
    bool direction = direction_char == '+' ? true : false;

    char digit1 = byte_data[4];
    char digit2 = byte_data[5];
    char digit3 = byte_data[6];
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
    setWheelMovement(motors[0].pwmOutput, motors[0].direction, pwm1, pwr1Fwd, pwr1Bwd);
    setWheelMovement(motors[1].pwmOutput, motors[1].direction, pwm2, pwr2Fwd, pwr2Bwd);
    setWheelMovement(motors[2].pwmOutput, motors[2].direction, pwm3, pwr3Fwd, pwr3Bwd);
    setWheelMovement(motors[3].pwmOutput, motors[3].direction, pwm4, pwr4Fwd, pwr4Bwd);

    delay(10);
}
