#include "sennep_logic.h"

typedef struct {
    int32_t def, min, max;
} param_def_t;

/* Defaults are first guesses — tune at the robot with SET_PARAM. */
static const param_def_t PARAMS[SL_NUM_PARAMS] = {
    [SP_PARAM_LINK_TIMEOUT_MS] = {200, 50, 2000},
    [SP_PARAM_RAMP_PERMILLE_PER_S] = {2500, 100, 20000},      /* 0 -> full in 0.4 s */
    [SP_PARAM_STOP_RAMP_PERMILLE_PER_S] = {6000, 100, 50000}, /* full -> 0 in ~0.17 s */
    [SP_PARAM_MAX_WHEEL_MM_S] = {800, 100, 3000},             /* rim speed at duty 1000 (no encoders: estimate) */
    [SP_PARAM_ROT_ARM_MM] = {450, 50, 2000},                  /* (track + wheelbase) / 2 */
    [SP_PARAM_TOF_STOP_MM] = {120, 0, 2000},                  /* 0 = off */
    [SP_PARAM_EDGE_ENABLE] = {1, 0, 1},
    [SP_PARAM_BATTERY_LOW_MV] = {10200, 0, 13000},            /* 3S: 3.4 V/cell. 0 = off */
    [SP_PARAM_MOTOR_INVERT_MASK] = {0, 0, 15},
};

/* Direction blocked by each sensor. */
static const uint8_t EDGE_BLOCK[4] = {
    SP_BLOCK_FWD | SP_BLOCK_LEFT | SP_BLOCK_ROTATE,  /* FL */
    SP_BLOCK_FWD | SP_BLOCK_RIGHT | SP_BLOCK_ROTATE, /* FR */
    SP_BLOCK_BACK | SP_BLOCK_LEFT | SP_BLOCK_ROTATE, /* RL */
    SP_BLOCK_BACK | SP_BLOCK_RIGHT | SP_BLOCK_ROTATE /* RR */
};
static const uint8_t TOF_BLOCK[8] = {
    SP_BLOCK_FWD, SP_BLOCK_FWD | SP_BLOCK_LEFT, SP_BLOCK_LEFT, SP_BLOCK_BACK | SP_BLOCK_LEFT,
    SP_BLOCK_BACK, SP_BLOCK_BACK | SP_BLOCK_RIGHT, SP_BLOCK_RIGHT, SP_BLOCK_FWD | SP_BLOCK_RIGHT};

static int32_t abs32(int32_t v) { return v < 0 ? -v : v; }

static void clear_command(sl_robot_t *r) {
    r->wheels_mode = false;
    r->vx = r->vy = r->wz = 0;
    for (uint8_t i = 0; i < 4; i++) r->wheel_cmd[i] = 0;
}

static void stop_now(sl_robot_t *r) {
    for (uint8_t i = 0; i < 4; i++) {
        r->duty_x1000[i] = 0;
        r->duty[i] = 0;
    }
}

void sl_init(sl_robot_t *r, uint32_t now_ms) {
    memset(r, 0, sizeof(*r));
    for (uint8_t i = 0; i < SL_NUM_PARAMS; i++) r->param[i] = PARAMS[i].def;
    r->state = SP_STATE_DISARMED;
    r->last_link_ms = now_ms;
    r->last_update_ms = now_ms;
}

uint8_t sl_arm(sl_robot_t *r, uint32_t now_ms) {
    r->last_link_ms = now_ms;
    if (r->estop_hw || r->state == SP_STATE_ESTOP || r->state == SP_STATE_FAULT) return SP_RESULT_REJECTED;
    clear_command(r);
    r->faults &= (uint16_t)~SP_FAULT_LINK_LOST;
    r->state = SP_STATE_ARMED;
    return SP_RESULT_OK;
}

uint8_t sl_disarm(sl_robot_t *r, uint32_t now_ms) {
    r->last_link_ms = now_ms;
    clear_command(r);
    r->faults &= (uint16_t)~(SP_FAULT_ESTOP_SW | SP_FAULT_LINK_LOST | SP_FAULT_BATTERY_LOW);
    r->battery_low_timing = false;
    if (r->estop_hw) return SP_RESULT_OK; /* stays in ESTOP until the button is released */
    r->state = SP_STATE_DISARMED;
    return SP_RESULT_OK;
}

uint8_t sl_estop_sw(sl_robot_t *r, uint32_t now_ms) {
    (void)now_ms;
    clear_command(r);
    r->faults |= SP_FAULT_ESTOP_SW;
    r->state = SP_STATE_ESTOP;
    stop_now(r);
    return SP_RESULT_OK;
}

void sl_heartbeat(sl_robot_t *r, uint32_t now_ms) { r->last_link_ms = now_ms; }

void sl_drive(sl_robot_t *r, int16_t vx, int16_t vy, int16_t wz, uint32_t now_ms) {
    r->last_link_ms = now_ms;
    if (r->state != SP_STATE_ARMED) return;
    r->wheels_mode = false;
    r->vx = vx;
    r->vy = vy;
    r->wz = wz;
}

void sl_wheels(sl_robot_t *r, const int16_t duty[4], uint32_t now_ms) {
    r->last_link_ms = now_ms;
    if (r->state != SP_STATE_ARMED) return;
    r->wheels_mode = true;
    for (uint8_t i = 0; i < 4; i++) {
        int16_t d = duty[i];
        r->wheel_cmd[i] = d > 1000 ? 1000 : (d < -1000 ? -1000 : d);
    }
}

uint8_t sl_set_param(sl_robot_t *r, uint8_t id, float value) {
    if (id >= SL_NUM_PARAMS || value != value) return SP_RESULT_BAD_PARAM; /* NaN check */
    if (value < (float)PARAMS[id].min || value > (float)PARAMS[id].max) return SP_RESULT_BAD_PARAM;
    r->param[id] = (int32_t)(value + (value >= 0 ? 0.5f : -0.5f));
    return SP_RESULT_OK;
}

uint8_t sl_get_param(const sl_robot_t *r, uint8_t id, float *value) {
    if (id >= SL_NUM_PARAMS) return SP_RESULT_BAD_PARAM;
    *value = (float)r->param[id];
    return SP_RESULT_OK;
}

void sl_mecanum(int16_t vx, int16_t vy, int16_t wz, int32_t max_wheel_mm_s, int32_t rot_arm_mm, int16_t duty[4]) {
    int32_t rot = (int32_t)wz * rot_arm_mm / 1000; /* mm/s at the wheel */
    int32_t w[4] = {
        (int32_t)vx - vy - rot, /* FL */
        (int32_t)vx + vy + rot, /* FR */
        (int32_t)vx + vy - rot, /* RL */
        (int32_t)vx - vy + rot  /* RR */
    };
    int32_t peak = max_wheel_mm_s;
    for (uint8_t i = 0; i < 4; i++)
        if (abs32(w[i]) > peak) peak = abs32(w[i]); /* keep the direction, reduce the speed */
    for (uint8_t i = 0; i < 4; i++) duty[i] = (int16_t)(w[i] * 1000 / peak);
}

static uint8_t compute_blocked(const sl_robot_t *r) {
    uint8_t b = 0;
    if (r->param[SP_PARAM_EDGE_ENABLE])
        for (uint8_t i = 0; i < 4; i++)
            if (r->edge_bits & (1u << i)) b |= EDGE_BLOCK[i];
    int32_t stop = r->param[SP_PARAM_TOF_STOP_MM];
    if (stop > 0)
        for (uint8_t i = 0; i < 8; i++)
            if (r->tof_mm[i] != 0 && r->tof_mm[i] < stop) b |= TOF_BLOCK[i];
    return b;
}

static void update_state(sl_robot_t *r, uint32_t now_ms) {
    if (r->estop_hw) {
        r->faults |= SP_FAULT_ESTOP_HW;
        r->state = SP_STATE_ESTOP;
    } else if (r->faults & SP_FAULT_ESTOP_HW) {
        r->faults &= (uint16_t)~SP_FAULT_ESTOP_HW;
        if (!(r->faults & SP_FAULT_ESTOP_SW)) r->state = SP_STATE_DISARMED; /* released: needs a new ARM */
    }

    int32_t low = r->param[SP_PARAM_BATTERY_LOW_MV];
    if (low > 0 && r->battery_mv != 0 && r->battery_mv < low) {
        if (!r->battery_low_timing) {
            r->battery_low_timing = true;
            r->battery_low_since_ms = now_ms;
        } else if ((uint32_t)(now_ms - r->battery_low_since_ms) >= SL_BATTERY_LOW_HOLD_MS) {
            r->faults |= SP_FAULT_BATTERY_LOW;
            if (r->state == SP_STATE_ARMED || r->state == SP_STATE_DISARMED || r->state == SP_STATE_LINK_LOST)
                r->state = SP_STATE_FAULT;
        }
    } else {
        r->battery_low_timing = false;
    }

    if (r->state == SP_STATE_ARMED && (uint32_t)(now_ms - r->last_link_ms) > (uint32_t)r->param[SP_PARAM_LINK_TIMEOUT_MS]) {
        r->faults |= SP_FAULT_LINK_LOST;
        r->state = SP_STATE_LINK_LOST;
    }
    if (r->state != SP_STATE_ARMED) clear_command(r);
}

void sl_update(sl_robot_t *r, uint32_t now_ms) {
    uint32_t dt = now_ms - r->last_update_ms;
    r->last_update_ms = now_ms;
    if (dt > 100) dt = 100; /* first call / long stall: don't jump */

    update_state(r, now_ms);
    r->blocked = compute_blocked(r);

    int16_t target[4] = {0, 0, 0, 0};
    bool hard_stop = r->edge_bits != 0 && r->param[SP_PARAM_EDGE_ENABLE];
    if (r->state == SP_STATE_ARMED) {
        if (r->wheels_mode) {
            /* raw wheel test mode: any block stops everything (wheel commands have no direction to filter) */
            if (!r->blocked)
                for (uint8_t i = 0; i < 4; i++) target[i] = r->wheel_cmd[i];
        } else {
            int16_t vx = r->vx, vy = r->vy, wz = r->wz;
            if ((r->blocked & SP_BLOCK_FWD) && vx > 0) vx = 0;
            if ((r->blocked & SP_BLOCK_BACK) && vx < 0) vx = 0;
            if ((r->blocked & SP_BLOCK_LEFT) && vy > 0) vy = 0;
            if ((r->blocked & SP_BLOCK_RIGHT) && vy < 0) vy = 0;
            if (r->blocked & SP_BLOCK_ROTATE) wz = 0;
            sl_mecanum(vx, vy, wz, r->param[SP_PARAM_MAX_WHEEL_MM_S], r->param[SP_PARAM_ROT_ARM_MM], target);
        }
    }

    if (r->state == SP_STATE_ESTOP) {
        stop_now(r);
    } else {
        int32_t up = r->param[SP_PARAM_RAMP_PERMILLE_PER_S] * (int32_t)dt;        /* x1000 units per tick */
        int32_t down = r->param[SP_PARAM_STOP_RAMP_PERMILLE_PER_S] * (int32_t)dt;
        for (uint8_t i = 0; i < 4; i++) {
            int32_t cur = r->duty_x1000[i];
            int32_t tgt = (int32_t)target[i] * 1000;
            int32_t goal = tgt, step = up;
            if ((cur > 0 && tgt < 0) || (cur < 0 && tgt > 0)) {
                goal = 0; /* reversing: brake to zero first, accelerate the other way on later ticks */
                step = down;
            } else if (abs32(tgt) < abs32(cur)) {
                step = down;
            }
            if (hard_stop && abs32(goal) < abs32(cur)) {
                cur = goal; /* edge sensor: no ramp when slowing down */
            } else if (cur < goal) {
                cur = (goal - cur > step) ? cur + step : goal;
            } else if (cur > goal) {
                cur = (cur - goal > step) ? cur - step : goal;
            }
            r->duty_x1000[i] = cur;
            r->duty[i] = (int16_t)(cur / 1000);
        }
    }

    bool moving = false;
    for (uint8_t i = 0; i < 4; i++)
        if (r->duty_x1000[i] != 0) moving = true;
    /* EN stays on while ramping down after DISARM/LINK_LOST, then drops */
    r->enable = r->state == SP_STATE_ARMED || (moving && r->state != SP_STATE_ESTOP);
}
