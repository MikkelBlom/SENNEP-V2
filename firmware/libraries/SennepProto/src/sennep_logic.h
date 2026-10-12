/*
 * SENNEP Mega control/safety logic (portable C99, no I/O).
 * The Arduino sketch feeds it inputs and applies its outputs; host tests drive it with fake time.
 *
 * Approved safety rules (docs/electronics/PI_MEGA_LINK.md §4, round 2):
 *  - motors only run in ARMED, which needs an explicit ARM from the Pi (a Mega reboot never drives by itself)
 *  - no CMD_DRIVE / CMD_WHEELS / HEARTBEAT for LINK_TIMEOUT_MS -> LINK_LOST: ramp to stop, then a new ARM is required
 *  - edge sensors and too-close ToF block motion towards them, whatever the Pi commands
 *  - hardware e-stop -> ESTOP immediately; released -> DISARMED (needs ARM)
 *
 * Wheel order everywhere: FL, FR, RL, RR. Body frame: x forward, y left, z up (wz > 0 = counter-clockwise).
 */
#ifndef SENNEP_LOGIC_H
#define SENNEP_LOGIC_H

#include "sennep_msgs.h"

#ifdef __cplusplus
extern "C" {
#endif

#define SL_NUM_PARAMS 9u
#define SL_BATTERY_LOW_HOLD_MS 2000u /* battery must be low this long before FAULT (load sag is normal) */

typedef struct {
    /* parameters, indexed by SP_PARAM_* */
    int32_t param[SL_NUM_PARAMS];

    /* state */
    uint8_t state;     /* SP_STATE_* */
    uint16_t faults;   /* SP_FAULT_* bits */
    uint8_t blocked;   /* SP_BLOCK_* bits, recomputed each update */
    uint32_t last_link_ms;
    uint32_t last_update_ms;
    uint32_t battery_low_since_ms;
    bool battery_low_timing;

    /* command */
    bool wheels_mode;  /* true: CMD_WHEELS, false: CMD_DRIVE */
    int16_t vx, vy, wz;
    int16_t wheel_cmd[4];

    /* inputs (set by the sketch before sl_update) */
    bool estop_hw;
    uint8_t edge_bits;
    uint16_t tof_mm[8];
    uint16_t battery_mv; /* 0 = unknown */

    /* outputs */
    int32_t duty_x1000[4]; /* ramped duty, permille * 1000 for sub-step precision */
    int16_t duty[4];       /* -1000..1000 to apply (before MOTOR_INVERT_MASK, which the sketch applies) */
    bool enable;           /* BTS7960 EN pins */
} sl_robot_t;

void sl_init(sl_robot_t *r, uint32_t now_ms);

/* Pi messages. Each returns an SP_RESULT_* code. */
uint8_t sl_arm(sl_robot_t *r, uint32_t now_ms);
uint8_t sl_disarm(sl_robot_t *r, uint32_t now_ms);
uint8_t sl_estop_sw(sl_robot_t *r, uint32_t now_ms);
void sl_heartbeat(sl_robot_t *r, uint32_t now_ms);
void sl_drive(sl_robot_t *r, int16_t vx_mm_s, int16_t vy_mm_s, int16_t wz_mrad_s, uint32_t now_ms);
void sl_wheels(sl_robot_t *r, const int16_t duty_permille[4], uint32_t now_ms);
uint8_t sl_set_param(sl_robot_t *r, uint8_t id, float value);
uint8_t sl_get_param(const sl_robot_t *r, uint8_t id, float *value);

/* Run state machine, blocking, kinematics and ramps. Call every loop (>= 100 Hz). */
void sl_update(sl_robot_t *r, uint32_t now_ms);

/* Mecanum inverse kinematics (exposed for tests). Writes duty -1000..1000, scaled down together if any exceeds. */
void sl_mecanum(int16_t vx, int16_t vy, int16_t wz, int32_t max_wheel_mm_s, int32_t rot_arm_mm, int16_t duty[4]);

#ifdef __cplusplus
}
#endif
#endif
