#include "hw.h"
#include "config.h"
#include <Wire.h>
#include <sennep_msgs.h>

namespace hw {

// ---------------- Motors ----------------
// Timers 3, 4, 5 in fast PWM mode 14 (TOP = ICRn) without prescaler: 16 MHz / 800 = 20 kHz (inaudible, BTS7960 max 25 kHz).
static const uint16_t PWM_TOP = 799;

static void setupTimer(volatile uint8_t &tccrA, volatile uint8_t &tccrB, volatile uint16_t &icr) {
    tccrA = _BV(WGM11);
    tccrB = _BV(WGM13) | _BV(WGM12) | _BV(CS10);
    icr = PWM_TOP;
}

// Map pin -> output-compare register and its COM bit in TCCRnA.
struct PwmOut { volatile uint16_t *ocr; volatile uint8_t *tccrA; uint8_t com; };

static PwmOut pwmFor(uint8_t pin) {
    switch (pin) {
        case 5: return {&OCR3A, &TCCR3A, _BV(COM3A1)};
        case 2: return {&OCR3B, &TCCR3A, _BV(COM3B1)};
        case 3: return {&OCR3C, &TCCR3A, _BV(COM3C1)};
        case 6: return {&OCR4A, &TCCR4A, _BV(COM4A1)};
        case 7: return {&OCR4B, &TCCR4A, _BV(COM4B1)};
        case 8: return {&OCR4C, &TCCR4A, _BV(COM4C1)};
        case 46: return {&OCR5A, &TCCR5A, _BV(COM5A1)};
        case 45: return {&OCR5B, &TCCR5A, _BV(COM5B1)};
        case 44: return {&OCR5C, &TCCR5A, _BV(COM5C1)};
        default: return {nullptr, nullptr, 0};
    }
}

static PwmOut rpwm[4], lpwm[4];

static void pwmWrite(const PwmOut &p, uint16_t permille) {
    if (!p.ocr) return;
    uint16_t v = (uint32_t)permille * PWM_TOP / 1000;
    if (v == 0) {
        *p.tccrA &= ~p.com; // disconnect: pin is a plain LOW output (no 1-tick glitch)
    } else {
        *p.ocr = v;
        *p.tccrA |= p.com;
    }
}

void motorsBegin() {
    digitalWrite(MOTOR_EN_PIN, LOW);
    pinMode(MOTOR_EN_PIN, OUTPUT);
    setupTimer(TCCR3A, TCCR3B, ICR3);
    setupTimer(TCCR4A, TCCR4B, ICR4);
    setupTimer(TCCR5A, TCCR5B, ICR5);
    for (uint8_t i = 0; i < 4; i++) {
        digitalWrite(MOTOR_RPWM[i], LOW);
        digitalWrite(MOTOR_LPWM[i], LOW);
        pinMode(MOTOR_RPWM[i], OUTPUT);
        pinMode(MOTOR_LPWM[i], OUTPUT);
        rpwm[i] = pwmFor(MOTOR_RPWM[i]);
        lpwm[i] = pwmFor(MOTOR_LPWM[i]);
        pwmWrite(rpwm[i], 0);
        pwmWrite(lpwm[i], 0);
    }
}

void motorsApply(const int16_t duty[4], bool enable, uint8_t invert_mask) {
    for (uint8_t i = 0; i < 4; i++) {
        int16_t d = enable ? duty[i] : 0;
        if (invert_mask & (1u << i)) d = -d;
        // Never drive both sides: set the inactive side to 0 first.
        if (d >= 0) {
            pwmWrite(lpwm[i], 0);
            pwmWrite(rpwm[i], (uint16_t)d);
        } else {
            pwmWrite(rpwm[i], 0);
            pwmWrite(lpwm[i], (uint16_t)(-d));
        }
    }
    digitalWrite(MOTOR_EN_PIN, enable ? HIGH : LOW);
}

void motorsReadCurrent(uint16_t raw[4]) {
    for (uint8_t i = 0; i < 4; i++) raw[i] = (uint16_t)analogRead(MOTOR_IS_PIN[i]); // ~110 us each
}

// ---------------- Safety inputs ----------------
void inputsBegin() {
    pinMode(ESTOP_SENSE_PIN, INPUT_PULLUP);
    for (uint8_t i = 0; i < 4; i++) pinMode(EDGE_PIN[i], INPUT_PULLUP);
    pinMode(BUZZER_PIN, OUTPUT);
    pinMode(HEADLIGHT_PIN, OUTPUT);
    pinMode(TAILLIGHT_PIN, OUTPUT);
}

bool estopPressed() { return digitalRead(ESTOP_SENSE_PIN) == HIGH; }

uint8_t edgeBits() {
    uint8_t b = 0;
    for (uint8_t i = 0; i < 4; i++)
        if (digitalRead(EDGE_PIN[i]) == HIGH) b |= (uint8_t)(1u << i);
    return b;
}

// ---------------- I2C ----------------
static const uint8_t MPU_ADDR = 0x68, QMC_ADDR = 0x0D, INA_ADDR = 0x40;

void i2cBegin() {
    Wire.begin();
    Wire.setClock(I2C_HZ);
    Wire.setWireTimeout(I2C_TIMEOUT_US, true); // a stuck bus must never freeze the control loop
}

static bool writeReg(uint8_t addr, uint8_t reg, uint8_t val) {
    Wire.beginTransmission(addr);
    Wire.write(reg);
    Wire.write(val);
    return Wire.endTransmission() == 0;
}

static bool readRegs(uint8_t addr, uint8_t reg, uint8_t *buf, uint8_t n) {
    Wire.beginTransmission(addr);
    Wire.write(reg);
    if (Wire.endTransmission(false) != 0) return false;
    if (Wire.requestFrom(addr, n) != n) return false;
    for (uint8_t i = 0; i < n; i++) buf[i] = (uint8_t)Wire.read();
    return true;
}

bool imuBegin() {
    // PWR_MGMT_1: wake, clock = gyro X PLL. Defaults: +-2 g, +-250 dps. DLPF 44 Hz (CONFIG=3).
    return writeReg(MPU_ADDR, 0x6B, 0x01) && writeReg(MPU_ADDR, 0x1A, 0x03);
}

bool imuRead(Imu &o) {
    uint8_t b[14];
    if (!readRegs(MPU_ADDR, 0x3B, b, 14)) return false; // big-endian: AX AY AZ TEMP GX GY GZ
    for (uint8_t i = 0; i < 3; i++) {
        o.acc[i] = (int16_t)((b[2 * i] << 8) | b[2 * i + 1]);
        o.gyro[i] = (int16_t)((b[8 + 2 * i] << 8) | b[9 + 2 * i]);
    }
    o.temp = (int16_t)((b[6] << 8) | b[7]);
    return true;
}

bool magBegin() {
    // SET/RESET period, then continuous mode, 200 Hz, 8 G, OSR 512.
    return writeReg(QMC_ADDR, 0x0B, 0x01) && writeReg(QMC_ADDR, 0x09, 0x1D);
}

bool magRead(Mag &o) {
    uint8_t st;
    if (!readRegs(QMC_ADDR, 0x06, &st, 1) || !(st & 0x01)) return false; // DRDY
    uint8_t b[6];
    if (!readRegs(QMC_ADDR, 0x00, b, 6)) return false; // little-endian X Y Z
    for (uint8_t i = 0; i < 3; i++) o.v[i] = (int16_t)(b[2 * i] | (b[2 * i + 1] << 8));
    return true;
}

bool inaBegin() {
    uint8_t b[2];
    return readRegs(INA_ADDR, 0x00, b, 2); // config register answers -> present (defaults: 32 V, +-320 mV)
}

bool inaRead(uint16_t &bus_mv, int16_t &shunt_ma) {
    uint8_t b[2];
    if (!readRegs(INA_ADDR, 0x02, b, 2)) return false;
    bus_mv = (uint16_t)(((b[0] << 8) | b[1]) >> 3) * 4; // 4 mV LSB
    if (!readRegs(INA_ADDR, 0x01, b, 2)) return false;
    // Shunt voltage, 10 uV LSB. Module shunt = 0.1 ohm -> 1 LSB = 0.1 mA, max ~3.2 A.
    // NOTE: the module's shunt cannot carry motor current; see PI_MEGA_LINK.md / circuit task.
    shunt_ma = (int16_t)((b[0] << 8) | b[1]) / 10;
    return true;
}

// ---------------- Outputs ----------------
void outputSet(uint8_t output, uint16_t value) {
    if (value > 1000) value = 1000;
    switch (output) {
        case SP_OUTPUT_HEADLIGHTS: analogWrite(HEADLIGHT_PIN, (int)((uint32_t)value * 255 / 1000)); break;
        case SP_OUTPUT_TAILLIGHTS: analogWrite(TAILLIGHT_PIN, (int)((uint32_t)value * 255 / 1000)); break;
        case SP_OUTPUT_BUZZER: digitalWrite(BUZZER_PIN, value ? HIGH : LOW); break;
        default: break;
    }
}

} // namespace hw
