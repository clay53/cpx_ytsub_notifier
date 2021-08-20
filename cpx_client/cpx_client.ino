#include <Adafruit_CircuitPlayground.h>

void setup() {
  CircuitPlayground.begin();
  // CircuitPlayground.speaker.begin();
  Serial.begin(9600);
  CircuitPlayground.setBrightness(3);
}

void loop() {
  if (Serial.available() > 0) {
    int incomingByte = Serial.read();

    switch(incomingByte) {
      case 43: // '+' gain subscriber
        for (int i = 0; i < 10; i++) {
          CircuitPlayground.setPixelColor(i, 255, 0, 0);
          CircuitPlayground.playTone(2000, 50, true);
        }
        for (int i = 9; i >= 0; i--) {
          CircuitPlayground.setPixelColor(i, 0, 0, 0);
          delay(25);
        }
        Serial.println("OH YAY SUBSCRIBER");
        break;
      case 45: // '-' lose subscriber
        for (int i = 9; i >= 0; i--) {
          CircuitPlayground.setPixelColor(i, 255, 0, 0);
          CircuitPlayground.playTone(500, 50, true);
        }
        for (int i = 0; i < 10; i++) {
          CircuitPlayground.setPixelColor(i, 0, 0, 0);
          delay(25);
        }
        Serial.println("OH NO NO SUBSCRIBER");
        break;
      default:
        Serial.println("OH NO IDK WTH YOU'RE TALKING ABOUT");
    }
  }
}
