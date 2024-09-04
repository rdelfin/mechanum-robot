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

bool rotDirection = false;
int wheel = 0;
bool increasing = true;
int pwmOutput = 0;

void setupWheel(int pwmPin, int fwdPin, int bwdPin) {
  pinMode(pwmPin, OUTPUT);
  pinMode(fwdPin, OUTPUT);
  pinMode(bwdPin, OUTPUT);
  // Set initial rotation direction
  digitalWrite(fwdPin, LOW);
  digitalWrite(fwdPin, HIGH);
}

void setup() {
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

  setWheelMovement(wheel == 0 ? pwmOutput : 0, rotDirection, pwm1, pwr1Fwd, pwr1Bwd);
  setWheelMovement(wheel == 1 ? pwmOutput : 0, rotDirection, pwm2, pwr2Fwd, pwr2Bwd);
  setWheelMovement(wheel == 2 ? pwmOutput : 0, rotDirection, pwm3, pwr3Fwd, pwr3Bwd);
  setWheelMovement(wheel == 3 ? pwmOutput : 0, rotDirection, pwm4, pwr4Fwd, pwr4Bwd);

  delay(20);
}
