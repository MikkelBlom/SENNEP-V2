// SENNEP V2 — Arduino Mega real-time hub.
// Design: docs/electronics/PI_MEGA_LINK.md. Protocol: protocol/MESSAGES.md. Build/test: firmware/README.md.
//
// loop() must never block: no delay(), no pulseIn(), no String. Everything is time-sliced.

#include <EEPROM.h>
#include <avr/wdt.h>
#include <sennep_logic.h>
#include <sennep_msgs.h>
#include "config.h"
#include "hw.h"
#include "link.h"

// ---- boot reason: MCUSR must be read (and the watchdog stopped) before anything else runs ----
uint8_t g_mcusr __attribute__((section(".noinit")));
void earlyInit() __attribute__((naked, used, section(".init3")));
void earlyInit() {
    g_mcusr = MCUSR;
    MCUSR = 0;
    wdt_disable();
}

static Link piLink(PI_SERIAL);
static Link node2Link(NODE2_SERIAL);
static Link node3Link(NODE3_SERIAL);

static sl_robot_t robot;
static bool piSeen = false;       // HELLO repeats until the Pi talks
static uint8_t bootReason = SP_BOOT_REASON_UNKNOWN;
static uint16_t bootCount = 0;
static bool imuOk, magOk, inaOk;
static uint16_t batteryMv = 0;
static int16_t currentMa = 0;
static uint16_t loopMaxUs = 0;
static hw::Imu imu;
static hw::Mag mag;

// ---------------- helpers ----------------
static void sendAck(uint8_t type, uint8_t seq, uint8_t result, float value = 0.0f) {
    sp_ack_t a = {type, seq, result, value};
    uint8_t p[SP_MAX_PAYLOAD];
    piLink.send(SP_MSG_ACK, p, sp_pack_ack(&a, p));
}

static void sendLog(uint8_t level, const char *text) {
    sp_log_t m;
    m.level = level;
    m.text_len = (uint8_t)min(strlen(text), sizeof(m.text));
    memcpy(m.text, text, m.text_len);
    uint8_t p[SP_MAX_PAYLOAD];
    piLink.send(SP_MSG_LOG, p, sp_pack_log(&m, p));
}

static void sendHello() {
    sp_hello_t h = {SP_PROTOCOL_VERSION, SP_SPEC_HASH, bootReason, bootCount, SP_MAX_PAYLOAD};
    uint8_t p[SP_MAX_PAYLOAD];
    piLink.send(SP_MSG_HELLO, p, sp_pack_hello(&h, p));
}

static Link *nodeLink(uint8_t port) {
    if (port == SP_PORT_NODE2) return &node2Link;
    if (port == SP_PORT_NODE3) return &node3Link;
    return nullptr;
}

// ---------------- incoming from Pi ----------------
static void handlePiFrame(const sp_frame_t &f, uint32_t nowMs, uint32_t nowUs) {
    piSeen = true;
    uint8_t res = SP_RESULT_OK;
    switch (f.type) {
        case SP_MSG_HEARTBEAT:
            sl_heartbeat(&robot, nowMs);
            return;
        case SP_MSG_ARM: res = sl_arm(&robot, nowMs); break;
        case SP_MSG_DISARM: res = sl_disarm(&robot, nowMs); break;
        case SP_MSG_ESTOP_SW: res = sl_estop_sw(&robot, nowMs); break;
        case SP_MSG_CMD_DRIVE: {
            sp_cmd_drive_t m;
            if (!sp_unpack_cmd_drive(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            sl_drive(&robot, m.vx_mm_s, m.vy_mm_s, m.wz_mrad_s, nowMs);
            return; // high-rate: no ACK
        }
        case SP_MSG_CMD_WHEELS: {
            sp_cmd_wheels_t m;
            if (!sp_unpack_cmd_wheels(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            sl_wheels(&robot, m.duty_permille, nowMs);
            return;
        }
        case SP_MSG_SET_PARAM: {
            sp_set_param_t m;
            if (!sp_unpack_set_param(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            res = sl_set_param(&robot, m.param, m.value);
            float v = 0;
            sl_get_param(&robot, m.param, &v);
            sendAck(f.type, f.seq, res, v);
            return;
        }
        case SP_MSG_GET_PARAM: {
            sp_get_param_t m;
            float v = 0;
            if (!sp_unpack_get_param(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            res = sl_get_param(&robot, m.param, &v);
            sendAck(f.type, f.seq, res, v);
            return;
        }
        case SP_MSG_SET_OUTPUT: {
            sp_set_output_t m;
            if (!sp_unpack_set_output(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            hw::outputSet(m.output, m.value);
            break;
        }
        case SP_MSG_TIME_SYNC: {
            sp_time_sync_t m;
            if (!sp_unpack_time_sync(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            sp_time_sync_reply_t r = {m.t_pi_us, nowUs};
            uint8_t p[SP_MAX_PAYLOAD];
            piLink.send(SP_MSG_TIME_SYNC_REPLY, p, sp_pack_time_sync_reply(&r, p));
            return;
        }
        case SP_MSG_NODE_SEND: {
            sp_node_send_t m;
            if (!sp_unpack_node_send(f.payload, f.len, &m)) { res = SP_RESULT_BAD_LENGTH; break; }
            Link *l = nodeLink(m.port);
            if (!l || m.frame_len < SP_HEADER + SP_CRC) { res = SP_RESULT_BAD_PARAM; break; }
            res = l->sendRaw(m.frame, m.frame_len) ? SP_RESULT_OK : SP_RESULT_REJECTED;
            break;
        }
        default: res = SP_RESULT_UNKNOWN_MSG; break;
    }
    sendAck(f.type, f.seq, res);
}

// A node frame is forwarded to the Pi unchanged (already CRC-checked by our decoder).
static void forwardNodeFrame(uint8_t port, const sp_frame_t &f) {
    sp_node_frame_t m;
    uint8_t rawLen = (uint8_t)(f.len + SP_HEADER + SP_CRC);
    if (rawLen > sizeof(m.frame)) return; // node payload > 55 bytes cannot be wrapped: node bug
    m.port = port;
    m.frame_len = rawLen;
    memcpy(m.frame, f.payload - SP_HEADER, rawLen);
    uint8_t p[SP_MAX_PAYLOAD];
    piLink.send(SP_MSG_NODE_FRAME, p, sp_pack_node_frame(&m, p));
}

// ---------------- telemetry ----------------
static uint32_t tImu, tMag, tMotors, tRanges, tStatus, tHello;

static bool due(uint32_t &last, uint32_t period, uint32_t now) {
    if ((uint32_t)(now - last) < period) return false;
    last = now;
    return true;
}

// At most one telemetry frame per loop pass, so the 64-byte TX buffer drains between frames.
static void telemetry(uint32_t nowMs, uint32_t nowUs) {
    uint8_t p[SP_MAX_PAYLOAD];
    if (!piSeen && due(tHello, HELLO_PERIOD_MS, nowMs)) {
        sendHello();
    } else if (imuOk && due(tImu, IMU_PERIOD_MS, nowMs)) {
        if (hw::imuRead(imu)) {
            sp_imu_t m;
            m.t_us = nowUs;
            memcpy(m.acc, imu.acc, sizeof(m.acc));
            memcpy(m.gyro, imu.gyro, sizeof(m.gyro));
            m.temp = imu.temp;
            piLink.send(SP_MSG_IMU, p, sp_pack_imu(&m, p));
        }
    } else if (magOk && due(tMag, MAG_PERIOD_MS, nowMs)) {
        if (hw::magRead(mag)) {
            sp_mag_t m;
            m.t_us = nowUs;
            memcpy(m.mag, mag.v, sizeof(m.mag));
            piLink.send(SP_MSG_MAG, p, sp_pack_mag(&m, p));
        }
    } else if (due(tMotors, MOTORS_PERIOD_MS, nowMs)) {
        sp_motors_t m;
        m.t_us = nowUs;
        memcpy(m.duty_permille, robot.duty, sizeof(m.duty_permille));
        hw::motorsReadCurrent(m.current_raw);
        piLink.send(SP_MSG_MOTORS, p, sp_pack_motors(&m, p));
    } else if (due(tRanges, RANGES_PERIOD_MS, nowMs)) {
        sp_ranges_t m;
        memset(&m, 0, sizeof(m));
        m.t_us = nowUs;
        memcpy(m.tof_mm, robot.tof_mm, sizeof(m.tof_mm)); // TODO: VL53L0X via TCA9548A
        m.edge_bits = robot.edge_bits;                    // TODO: HC-SR04 -> sonar_mm
        piLink.send(SP_MSG_RANGES, p, sp_pack_ranges(&m, p));
    } else if (due(tStatus, STATUS_PERIOD_MS, nowMs)) {
        if (inaOk) inaOk = hw::inaRead(batteryMv, currentMa);
        const sp_decoder_t &d = piLink.decoder();
        uint16_t faults = robot.faults;
        if (!imuOk) faults |= SP_FAULT_IMU_MISSING;
        if (!magOk) faults |= SP_FAULT_MAG_MISSING;
        if (!inaOk) faults |= SP_FAULT_INA_MISSING;
        if (loopMaxUs > 5000) faults |= SP_FAULT_LOOP_OVERRUN;
        if (piLink.rxOverflowed()) faults |= SP_FAULT_RX_OVERFLOW;
        sp_status_t m = {nowUs, robot.state, faults, robot.blocked, batteryMv, currentMa,
                         loopMaxUs, d.crc_errors, d.frame_errors, d.seq_gaps};
        piLink.send(SP_MSG_STATUS, p, sp_pack_status(&m, p));
        loopMaxUs = 0;
    }
}

// ---------------- setup / loop ----------------
void setup() {
    hw::motorsBegin(); // first: drivers off, PWM low
    hw::inputsBegin();

    if (g_mcusr & _BV(WDRF)) bootReason = SP_BOOT_REASON_WATCHDOG;
    else if (g_mcusr & _BV(BORF)) bootReason = SP_BOOT_REASON_BROWN_OUT;
    else if (g_mcusr & _BV(EXTRF)) bootReason = SP_BOOT_REASON_EXTERNAL;
    else if (g_mcusr & _BV(PORF)) bootReason = SP_BOOT_REASON_POWER_ON;
    EEPROM.get(0, bootCount);
    bootCount++;
    EEPROM.put(0, bootCount);

    DEBUG_SERIAL.begin(115200);
    DEBUG_SERIAL.print(F("SENNEP Mega proto v"));
    DEBUG_SERIAL.print(SP_PROTOCOL_VERSION);
    DEBUG_SERIAL.print(F(" spec 0x"));
    DEBUG_SERIAL.print(SP_SPEC_HASH, HEX);
    DEBUG_SERIAL.print(F(" boot "));
    DEBUG_SERIAL.println(bootCount);

    piLink.begin(PI_BAUD);
    node2Link.begin(NODE_BAUD);
    node3Link.begin(NODE_BAUD);

    hw::i2cBegin();
    imuOk = hw::imuBegin();
    magOk = hw::magBegin();
    inaOk = hw::inaBegin();

    sl_init(&robot, millis());
    sendHello();
    if (bootReason == SP_BOOT_REASON_WATCHDOG) sendLog(2, "watchdog reset");
#if USE_AVR_WDT
    wdt_enable(WDTO_250MS);
#endif
}

void loop() {
    uint32_t startUs = micros();
    uint32_t nowMs = millis();

    sp_frame_t f;
    while (piLink.poll(f)) handlePiFrame(f, nowMs, micros());
    while (node2Link.poll(f)) forwardNodeFrame(SP_PORT_NODE2, f);
    while (node3Link.poll(f)) forwardNodeFrame(SP_PORT_NODE3, f);

    robot.estop_hw = hw::estopPressed();
    robot.edge_bits = hw::edgeBits();
    robot.battery_mv = inaOk ? batteryMv : 0;
    // robot.tof_mm[] filled by the ToF module when implemented (0 = no reading = no blocking)

    sl_update(&robot, nowMs);
    hw::motorsApply(robot.duty, robot.enable, (uint8_t)robot.param[SP_PARAM_MOTOR_INVERT_MASK]);

    telemetry(nowMs, micros());

#if USE_AVR_WDT
    wdt_reset();
#endif
    uint32_t dt = micros() - startUs;
    if (dt > loopMaxUs) loopMaxUs = dt > 0xFFFF ? 0xFFFF : (uint16_t)dt;
}
