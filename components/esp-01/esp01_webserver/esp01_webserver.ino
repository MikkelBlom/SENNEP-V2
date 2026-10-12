// Arduino Mega + ESP-01 (stock AT firmware) as a tiny web server.
// Joins your WiFi, prints its IP address in the Serial Monitor, and serves a page
// with uptime and the reading on A0. Open http://<ip>/ in a browser on the same network.
// Setup: copy secrets.h.example to secrets.h and fill in SSID/password.
// Wiring: see ../README.md. Run esp01_passthrough first if the module does not answer.

#include "secrets.h"

#define ESP Serial1
const long ESP_BAUD = 115200;  // the baud rate esp01_passthrough found

// Reads from the ESP until `expect` shows up or the timeout expires. Echoes everything to USB
// so you can see what the module says.
bool waitFor(const char* expect, unsigned long timeoutMs) {
  const size_t len = strlen(expect);
  size_t matched = 0;
  const unsigned long start = millis();
  while (millis() - start < timeoutMs) {
    if (!ESP.available()) continue;
    const char c = ESP.read();
    Serial.write(c);
    if (c == expect[matched]) {
      if (++matched == len) return true;
    } else {
      matched = (c == expect[0]) ? 1 : 0;
    }
  }
  return false;
}

bool sendCommand(const __FlashStringHelper* cmd, const char* expect, unsigned long timeoutMs) {
  ESP.println(cmd);
  return waitFor(expect, timeoutMs);
}

void halt(const __FlashStringHelper* reason) {
  Serial.println();
  Serial.println(reason);
  while (true) {}
}

void setup() {
  Serial.begin(115200);
  ESP.begin(ESP_BAUD);
  delay(1000);

  if (!sendCommand(F("AT"), "OK", 2000)) halt(F("Intet svar fra ESP-01. Kør esp01_passthrough for at finde fejlen."));
  sendCommand(F("AT+CWMODE=1"), "OK", 2000);  // station mode: join an existing network

  ESP.print(F("AT+CWJAP=\""));
  ESP.print(F(WIFI_SSID));
  ESP.print(F("\",\""));
  ESP.print(F(WIFI_PASSWORD));
  ESP.println('"');
  if (!waitFor("OK", 20000)) halt(F("Kunne ikke forbinde til WiFi. Tjek secrets.h (ESP-01 kan kun 2,4 GHz)."));

  sendCommand(F("AT+CIFSR"), "OK", 3000);         // prints the IP address
  sendCommand(F("AT+CIPMUX=1"), "OK", 2000);      // several connections (required by the server)
  sendCommand(F("AT+CIPSERVER=1,80"), "OK", 2000);
  Serial.println(F("\nServer kører. Åbn IP-adressen ovenfor (STAIP) i en browser."));
}

// Discards the rest of the HTTP request; we answer every path with the same page.
void drainInput() {
  unsigned long lastByte = millis();
  while (millis() - lastByte < 20) {
    if (ESP.available()) {
      ESP.read();
      lastByte = millis();
    }
  }
}

void loop() {
  ESP.setTimeout(1000);
  if (!ESP.find((char*)"+IPD,")) return;
  const int connectionId = ESP.parseInt();
  drainInput();

  const String body = String(F("<h1>SENNEP V2</h1><p>Oppetid: ")) + (millis() / 1000) +
                      F(" s</p><p>A0: ") + analogRead(A0) + F("</p>");
  const String response = String(F("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n"
                                   "Connection: close\r\nContent-Length: ")) +
                          body.length() + F("\r\n\r\n") + body;

  ESP.print(F("AT+CIPSEND="));
  ESP.print(connectionId);
  ESP.print(',');
  ESP.println(response.length());
  if (waitFor(">", 2000)) {
    ESP.print(response);
    waitFor("SEND OK", 5000);
  }
  ESP.print(F("AT+CIPCLOSE="));
  ESP.println(connectionId);
  waitFor("OK", 2000);
}
