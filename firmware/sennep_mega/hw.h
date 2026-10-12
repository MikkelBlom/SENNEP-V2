// Hardware glue: motors (BTS7960), safety inputs, I2C sensors. Kept thin — decisions live in sennep_logic.c.
#pragma once
#include <Arduino.h>

namespace hw {

void motorsBegin();
// duty -1000..1000 per wheel (FL, FR, RL, RR); invert_mask bit i flips wheel i.
void motorsApply(const int16_t duty[4], bool enable, uint8_t invert_mask);
void motorsReadCurrent(uint16_t raw[4]);

void inputsBegin();
bool estopPressed();
uint8_t edgeBits(); // bit i = no floor at corner i

struct Imu { int16_t acc[3], gyro[3], temp; };
struct Mag { int16_t v[3]; };

void i2cBegin();
bool imuBegin();
bool imuRead(Imu &out);
bool magBegin();
bool magRead(Mag &out);   // false if no new sample
bool inaBegin();
bool inaRead(uint16_t &bus_mv, int16_t &shunt_ma);

void outputSet(uint8_t output, uint16_t value); // value 0..1000

} // namespace hw
