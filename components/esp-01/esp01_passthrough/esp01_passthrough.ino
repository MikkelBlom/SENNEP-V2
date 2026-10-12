// ESP-01 passthrough for Arduino Mega.
// Finds the ESP-01's baud rate, then forwards everything between the Serial Monitor
// and the ESP-01 so you can type AT commands by hand.
// Wiring: see ../README.md (ESP TX -> Mega pin 19 RX1, ESP RX <- Mega pin 18 TX1 via level converter).
// Serial Monitor: 115200 baud, line ending "Both NL & CR".

const long BAUD_RATES[] = {115200, 9600, 57600, 74880};

bool probe(long baud) {
  Serial1.begin(baud);
  delay(100);
  while (Serial1.available()) Serial1.read();
  Serial1.print("AT\r\n");
  Serial1.setTimeout(1000);
  return Serial1.find((char*)"OK");
}

void setup() {
  Serial.begin(115200);
  Serial.println(F("Leder efter ESP-01 ..."));
  for (long baud : BAUD_RATES) {
    if (probe(baud)) {
      Serial.print(F("Svar ved "));
      Serial.print(baud);
      Serial.println(F(" baud. Prøv: AT+GMR (firmwareversion), AT+CWLAP (scan WiFi-net)."));
      return;
    }
    Serial1.end();
  }
  Serial.println(F("Intet svar. Tjek 3,3 V-forsyningen, at EN er høj, og om TX/RX er byttet om."));
  Serial1.begin(115200);
}

void loop() {
  while (Serial.available()) Serial1.write(Serial.read());
  while (Serial1.available()) Serial.write(Serial1.read());
}
