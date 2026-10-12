// SENNEP Mega — pins and rates.
// PIN MAP IS A DRAFT: the circuit task (docs/electronics/) owns the physical wiring. Change pins here only,
// and keep docs/electronics/PI_MEGA_LINK.md §4c (pin budget) in sync.
#pragma once
#include <Arduino.h>

// ---- Links ----
#define PI_SERIAL Serial1          // TX1 18 -> BSS138 -> Pi GPIO15 (RX); RX1 19 <- BSS138 <- Pi GPIO14 (TX)
#define PI_BAUD 250000UL           // exact on 16 MHz; try 500000 after measuring the level shifter
#define NODE2_SERIAL Serial2       // TX2 16 / RX2 17: expansion node #2 (free)
#define NODE3_SERIAL Serial3       // TX3 14 / RX3 15: LED node (Nano/Uno with WS2812B)
#define NODE_BAUD 250000UL
#define DEBUG_SERIAL Serial        // USB, 115200, human-readable boot line only

// ---- Motors: BTS7960, 20 kHz PWM on 16-bit timers 3/4/5 (timer 0 = millis, untouched) ----
// Wheel order FL, FR, RL, RR. RPWM = forward, LPWM = reverse (flip with PARAM MOTOR_INVERT_MASK, no rewiring needed).
static const uint8_t MOTOR_RPWM[4] = {2, 5, 7, 44};  // OC3B, OC3A, OC4B, OC5C
static const uint8_t MOTOR_LPWM[4] = {3, 6, 8, 45};  // OC3C, OC4A, OC4C, OC5B
#define MOTOR_EN_PIN 22                               // all R_EN + L_EN tied together; LOW = drivers off
static const uint8_t MOTOR_IS_PIN[4] = {A0, A1, A2, A3}; // R_IS + L_IS joined per driver (via resistor, see circuit)

// ---- Safety inputs (fail-safe: a broken wire reads as "danger") ----
#define ESTOP_SENSE_PIN 23          // INPUT_PULLUP; NC contact to GND. HIGH = e-stop pressed OR wire broken
static const uint8_t EDGE_PIN[4] = {24, 25, 26, 27}; // E18-D80NK, INPUT_PULLUP; LOW = floor seen, HIGH = no floor / broken

// ---- Reserved for modules not implemented yet (do not reuse) ----
#define SONAR_TRIG_PIN 28                             // shared trigger for 4x HC-SR04
static const uint8_t SONAR_ECHO_PIN[4] = {A8, A9, A10, A11}; // PCINT16-19
#define NRF_CE_PIN 48
#define NRF_CSN_PIN 53                                // + SPI 50/51/52
#define DOCK_IR_FIRST_PIN 31                          // 31..34: 4x VS1838B
#define RESET_FROM_PI "RESET pin"                     // Pi GPIO4 -> BSS138 ch3 -> RESET (flash from Pi)
// From docs/electronics/BREADBOARD_PLAN.md §2.4/§2.7 (consolidation 2026-10-12). Not read by the firmware yet.
#define MOTOR_BUS_SENSE_PIN A4                        // motor bus after the relay, divider 47k/10k (12.6 V -> 2.2 V)
#define ARM_BUTTON_PIN 29                             // red PBS-110, INPUT_PULLUP. Role open: ARM must come from the Pi

// ---- Simple outputs (SET_OUTPUT) ----
#define HEADLIGHT_PIN 9            // PWM (timer 2) -> MOSFET gate driver (IRFZ44N is not logic level!)
#define TAILLIGHT_PIN 10           // PWM (timer 2)
#define BUZZER_PIN 30              // active buzzer, on/off

// ---- I2C (20 SDA / 21 SCL): MPU-6050 0x68, QMC5883L 0x0D, INA219 0x40, TCA9548A 0x70 ----
#define I2C_HZ 400000UL
#define I2C_TIMEOUT_US 3000

// ---- Telemetry periods (ms) ----
#define IMU_PERIOD_MS 10
#define MAG_PERIOD_MS 20
#define MOTORS_PERIOD_MS 20
#define RANGES_PERIOD_MS 40
#define STATUS_PERIOD_MS 100
#define HELLO_PERIOD_MS 500

// ---- AVR hardware watchdog ----
// Old Mega 2560 bootloaders (some clones) hang in a reset loop after a watchdog reset.
// Test once at the robot (see firmware/README.md); set to 0 if the board never comes back.
#define USE_AVR_WDT 1
