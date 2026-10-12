/* Host tests for the portable C core: framing (COBS/CRC/stream decoder) and the approved safety logic.
 * Build + run: tests/run_all.sh */
#include <stdio.h>
#include <stdlib.h>
#include "sennep_logic.h"
#include "sennep_msgs.h"

static int fails = 0, checks = 0;
#define CHECK(c) do { checks++; if (!(c)) { fails++; printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); } } while (0)

static uint32_t rng = 12345;
static uint32_t rnd(void) { rng = rng * 1103515245u + 12345u; return rng >> 8; }

/* ---------------------------------------------------------------- framing */
static void test_crc(void) {
    CHECK(sp_crc16((const uint8_t *)"123456789", 9) == 0x29B1);
    CHECK(sp_crc16(NULL, 0) == 0xFFFF);
}

static void test_cobs_roundtrip(void) {
    uint8_t in[600], enc[620], dec[620];
    for (int t = 0; t < 3000; t++) {
        size_t n = rnd() % 600;
        int mode = t % 4; /* random, all zero, no zero (long runs >254), sparse zeros */
        for (size_t i = 0; i < n; i++)
            in[i] = mode == 0 ? (uint8_t)rnd() : mode == 1 ? 0 : mode == 2 ? (uint8_t)(1 + rnd() % 255) : (rnd() % 50 ? 7 : 0);
        size_t e = sp_cobs_encode(in, n, enc);
        CHECK(e <= n + n / 254 + 1);
        for (size_t i = 0; i < e; i++) if (enc[i] == 0) { CHECK(!"zero in COBS output"); break; }
        size_t d = sp_cobs_decode(enc, e, dec);
        CHECK(d == n && memcmp(in, dec, n) == 0);
    }
    /* known vectors (Wikipedia) */
    const uint8_t a[] = {0x00}, ea[] = {0x01, 0x01};
    const uint8_t b[] = {0x11, 0x22, 0x00, 0x33}, eb[] = {0x03, 0x11, 0x22, 0x02, 0x33};
    uint8_t o[8];
    CHECK(sp_cobs_encode(a, 1, o) == 2 && memcmp(o, ea, 2) == 0);
    CHECK(sp_cobs_encode(b, 4, o) == 5 && memcmp(o, eb, 5) == 0);
}

static int feed_all(sp_decoder_t *d, const uint8_t *s, size_t n, sp_frame_t *last) {
    int got = 0;
    for (size_t i = 0; i < n; i++)
        if (sp_decoder_feed(d, s[i], last)) got++;
    return got;
}

static void test_frame_roundtrip(void) {
    uint8_t wire[SP_MAX_WIRE], payload[SP_MAX_PAYLOAD];
    sp_decoder_t d;
    sp_decoder_init(&d);
    for (int t = 0; t < 2000; t++) {
        uint8_t len = (uint8_t)(rnd() % (SP_MAX_PAYLOAD + 1));
        for (uint8_t i = 0; i < len; i++) payload[i] = (uint8_t)(rnd() % 3 ? rnd() : 0);
        size_t n = sp_frame_encode((uint8_t)rnd(), (uint8_t)t, payload, len, wire);
        CHECK(n > 0 && n <= SP_MAX_WIRE && wire[n - 1] == 0);
        sp_frame_t f;
        CHECK(feed_all(&d, wire, n, &f) == 1);
        CHECK(f.len == len && f.seq == (uint8_t)t && memcmp(f.payload, payload, len) == 0);
    }
    CHECK(d.seq_gaps == 0 && d.crc_errors == 0 && d.frame_errors == 0);
    CHECK(sp_frame_encode(1, 0, payload, SP_MAX_PAYLOAD + 1, wire) == 0);
}

/* Corrupt a stream of N frames in different ways; every untouched frame must still arrive. */
static void test_stream_recovery(void) {
    enum { N = 400 };
    static uint8_t stream[N * SP_MAX_WIRE + 64];
    uint8_t payload[SP_MAX_PAYLOAD], wire[SP_MAX_WIRE];
    for (int mode = 0; mode < 4; mode++) {
        size_t pos = 0;
        int intact = 0;
        /* leading garbage, as after a reboot */
        for (int i = 0; i < 37; i++) stream[pos++] = (uint8_t)(rnd() | 1);
        for (int k = 0; k < N; k++) {
            uint8_t len = (uint8_t)(rnd() % 30);
            for (uint8_t i = 0; i < len; i++) payload[i] = (uint8_t)rnd();
            size_t n = sp_frame_encode(0x82, (uint8_t)k, payload, len, wire);
            bool damage = k % 3 == 1;
            if (damage && mode == 0 && n > 2) { /* drop one byte (not the delimiter) */
                size_t at = rnd() % (n - 1);
                memmove(wire + at, wire + at + 1, n - at - 1);
                n--;
            } else if (damage && mode == 1) { /* flip one bit */
                wire[rnd() % (n - 1)] ^= (uint8_t)(1u << (rnd() % 8));
            } else if (damage && mode == 2) { /* insert a random byte */
                size_t at = rnd() % (n - 1);
                memmove(wire + at + 1, wire + at, n - at);
                wire[at] = (uint8_t)rnd();
                n++;
            } else if (damage && mode == 3) { /* lose the delimiter: two frames merge, both are lost */
                n--;
            } else {
                intact++;
            }
            memcpy(stream + pos, wire, n);
            pos += n;
        }
        sp_decoder_t d;
        sp_decoder_init(&d);
        sp_frame_t f;
        int got = feed_all(&d, stream, pos, &f);
        /* -1: the leading garbage merges with frame 0 (no delimiter before it). Mode 3: merged partner frames are lost too. */
        int expect_min = (mode == 3 ? intact - N / 3 : intact) - 1;
        CHECK(got >= expect_min);
        CHECK(got <= intact + 1); /* corrupted frames must not pass (1-in-65536 CRC luck allowed once) */
        CHECK(d.crc_errors + d.frame_errors > 0);
        printf("  stream mode %d: %d/%d intact frames received, crc_err %u frame_err %u gaps %u\n", mode, got, intact,
               d.crc_errors, d.frame_errors, d.seq_gaps);
    }
}

static void test_overflow(void) {
    sp_decoder_t d;
    sp_decoder_init(&d);
    sp_frame_t f;
    for (int i = 0; i < 500; i++) sp_decoder_feed(&d, 0x55, &f); /* no delimiter for a long time */
    CHECK(!sp_decoder_feed(&d, 0, &f));
    CHECK(d.frame_errors == 1);
    uint8_t wire[SP_MAX_WIRE];
    size_t n = sp_frame_encode(1, 0, NULL, 0, wire);
    CHECK(feed_all(&d, wire, n, &f) == 1 && f.type == 1 && f.len == 0);
    /* back-to-back delimiters are ignored, not errors */
    sp_decoder_feed(&d, 0, &f);
    sp_decoder_feed(&d, 0, &f);
    CHECK(d.frame_errors == 1);
}

static void test_messages(void) {
    sp_cmd_drive_t a = {-300, 250, -1200, 7}, b;
    uint8_t p[SP_MAX_PAYLOAD];
    uint8_t n = sp_pack_cmd_drive(&a, p);
    CHECK(n == SP_SIZE_CMD_DRIVE);
    CHECK(sp_unpack_cmd_drive(p, n, &b) && b.vx_mm_s == -300 && b.vy_mm_s == 250 && b.wz_mrad_s == -1200 && b.flags == 7);
    CHECK(!sp_unpack_cmd_drive(p, (uint8_t)(n - 1), &b));
    sp_ack_t k = {0x20, 9, 0, 1.5f}, k2;
    n = sp_pack_ack(&k, p);
    CHECK(sp_unpack_ack(p, n, &k2) && k2.value == 1.5f);
    sp_node_frame_t nf, nf2;
    nf.port = 3;
    nf.frame_len = 5;
    memcpy(nf.frame, "\x01\x02\x03\x04\x05", 5);
    n = sp_pack_node_frame(&nf, p);
    CHECK(n == 6 && sp_unpack_node_frame(p, n, &nf2) && nf2.frame_len == 5 && nf2.frame[4] == 5);
    CHECK(SP_SIZE_RANGES <= SP_MAX_PAYLOAD);
}

/* ---------------------------------------------------------------- safety logic */
static uint32_t now;
static void run(sl_robot_t *r, uint32_t ms) { /* advance time in 5 ms loop steps */
    for (uint32_t t = 0; t < ms; t += 5) { now += 5; sl_update(r, now); }
}
static void drive_for(sl_robot_t *r, int16_t vx, int16_t vy, int16_t wz, uint32_t ms) {
    for (uint32_t t = 0; t < ms; t += 20) { sl_drive(r, vx, vy, wz, now); run(r, 20); }
}
static bool all_zero(const sl_robot_t *r) { return !r->duty[0] && !r->duty[1] && !r->duty[2] && !r->duty[3]; }

static void test_no_motion_without_arm(void) {
    sl_robot_t r;
    now = 1000;
    sl_init(&r, now);
    drive_for(&r, 500, 0, 0, 1000);
    CHECK(r.state == SP_STATE_DISARMED && all_zero(&r) && !r.enable);
    CHECK(sl_arm(&r, now) == SP_RESULT_OK);
    drive_for(&r, 500, 0, 0, 1000);
    CHECK(r.state == SP_STATE_ARMED && r.enable && r.duty[0] > 0 && r.duty[0] == r.duty[1]);
}

static void test_watchdog(void) {
    sl_robot_t r;
    now = 5000;
    sl_init(&r, now);
    sl_arm(&r, now);
    drive_for(&r, 400, 0, 0, 1000);
    CHECK(r.duty[0] == 500); /* 400 / 800 mm/s */
    uint32_t silent = r.last_link_ms; /* last command */
    while (r.state == SP_STATE_ARMED && now - silent < 1000) run(&r, 5);
    uint32_t detect = now - silent;
    CHECK(r.state == SP_STATE_LINK_LOST && detect > 200 && detect <= 210);
    while (!all_zero(&r) && now - silent < 2000) run(&r, 5);
    CHECK(all_zero(&r) && now - silent <= 200 + 100); /* 500 permille at 6000/s = 83 ms ramp */
    run(&r, 10);
    CHECK(!r.enable);
    /* link comes back: still no motion until a new ARM */
    drive_for(&r, 400, 0, 0, 500);
    CHECK(r.state == SP_STATE_LINK_LOST && all_zero(&r));
    CHECK(sl_arm(&r, now) == SP_RESULT_OK);
    drive_for(&r, 400, 0, 0, 500);
    CHECK(r.state == SP_STATE_ARMED && r.duty[0] == 500);
    /* heartbeats alone keep it armed but stopped */
    for (int i = 0; i < 50; i++) { sl_heartbeat(&r, now); run(&r, 20); }
    CHECK(r.state == SP_STATE_ARMED);
}

static void test_estop(void) {
    sl_robot_t r;
    now = 100;
    sl_init(&r, now);
    sl_arm(&r, now);
    drive_for(&r, 800, 0, 0, 1000);
    r.estop_hw = true;
    run(&r, 5);
    CHECK(r.state == SP_STATE_ESTOP && all_zero(&r) && !r.enable);
    CHECK(sl_arm(&r, now) == SP_RESULT_REJECTED);
    r.estop_hw = false;
    run(&r, 5);
    CHECK(r.state == SP_STATE_DISARMED && all_zero(&r));
    /* software e-stop latches until DISARM */
    sl_arm(&r, now);
    drive_for(&r, 800, 0, 0, 300);
    sl_estop_sw(&r, now);
    CHECK(all_zero(&r));
    CHECK(sl_arm(&r, now) == SP_RESULT_REJECTED);
    sl_disarm(&r, now);
    CHECK(sl_arm(&r, now) == SP_RESULT_OK);
}

static void test_edge_and_tof_blocking(void) {
    sl_robot_t r;
    now = 100;
    sl_init(&r, now);
    sl_arm(&r, now);
    drive_for(&r, 400, 0, 0, 500);
    CHECK(r.duty[0] == 500);
    r.edge_bits = 0x1; /* front-left corner sees no floor */
    sl_drive(&r, 400, 0, 0, now);
    run(&r, 5);
    CHECK(all_zero(&r)); /* immediate, no ramp */
    CHECK(r.blocked & SP_BLOCK_FWD);
    drive_for(&r, -300, 0, 0, 500); /* backing away is allowed */
    CHECK(r.duty[0] < 0);
    drive_for(&r, 0, 300, 0, 300); /* left is blocked by the FL corner */
    CHECK(all_zero(&r));
    drive_for(&r, 0, 0, 1000, 300); /* no rotation near an edge */
    CHECK(all_zero(&r));
    r.edge_bits = 0;
    r.tof_mm[4] = 80; /* something close behind */
    drive_for(&r, -300, 0, 0, 500);
    CHECK(all_zero(&r) && (r.blocked & SP_BLOCK_BACK));
    drive_for(&r, 300, 0, 0, 500);
    CHECK(r.duty[0] > 0);
    r.tof_mm[4] = 0; /* 0 = no reading = no block */
    drive_for(&r, -300, 0, 0, 500);
    CHECK(r.duty[0] < 0);
}

static void test_battery(void) {
    sl_robot_t r;
    now = 100;
    sl_init(&r, now);
    sl_arm(&r, now);
    r.battery_mv = 9800;
    drive_for(&r, 200, 0, 0, 1500); /* short sag: still armed */
    CHECK(r.state == SP_STATE_ARMED);
    drive_for(&r, 200, 0, 0, 700);
    CHECK(r.state == SP_STATE_FAULT && (r.faults & SP_FAULT_BATTERY_LOW));
    run(&r, 300);
    CHECK(all_zero(&r));
    CHECK(sl_arm(&r, now) == SP_RESULT_REJECTED);
}

static void test_kinematics_and_ramp(void) {
    int16_t d[4];
    sl_mecanum(400, 0, 0, 800, 450, d);
    CHECK(d[0] == 500 && d[1] == 500 && d[2] == 500 && d[3] == 500);
    sl_mecanum(0, 400, 0, 800, 450, d); /* strafe left: FL-, FR+, RL+, RR- */
    CHECK(d[0] == -500 && d[1] == 500 && d[2] == 500 && d[3] == -500);
    sl_mecanum(0, 0, 1000, 800, 400, d); /* CCW: left wheels back, right wheels forward */
    CHECK(d[0] == -500 && d[1] == 500 && d[2] == -500 && d[3] == 500);
    sl_mecanum(800, 800, 0, 800, 450, d); /* saturates: scaled together */
    CHECK(d[1] == 1000 && d[2] == 1000 && d[0] == 0 && d[3] == 0);

    sl_robot_t r;
    now = 0;
    sl_init(&r, now);
    sl_arm(&r, now);
    sl_drive(&r, 800, 0, 0, now);
    run(&r, 200); /* 2500 permille/s -> 500 after 200 ms */
    CHECK(r.duty[0] >= 480 && r.duty[0] <= 510);
    drive_for(&r, 800, 0, 0, 400);
    CHECK(r.duty[0] == 1000);
    /* reversal brakes at the stop rate, passes zero, then accelerates at the normal rate */
    sl_drive(&r, -800, 0, 0, now);
    run(&r, 100);
    CHECK(r.duty[0] >= 390 && r.duty[0] <= 410); /* 1000 - 6000*0.1 */
    drive_for(&r, -800, 0, 0, 1000);
    CHECK(r.duty[0] == -1000);
    CHECK(sl_set_param(&r, SP_PARAM_LINK_TIMEOUT_MS, 5000.f) == SP_RESULT_BAD_PARAM);
    CHECK(sl_set_param(&r, SP_PARAM_LINK_TIMEOUT_MS, 300.f) == SP_RESULT_OK && r.param[0] == 300);
    CHECK(sl_set_param(&r, 99, 1.f) == SP_RESULT_BAD_PARAM);
}

static void test_wheels_mode(void) {
    sl_robot_t r;
    now = 0;
    sl_init(&r, now);
    sl_arm(&r, now);
    int16_t w[4] = {300, 0, -300, 2000};
    for (int i = 0; i < 50; i++) { sl_wheels(&r, w, now); run(&r, 20); }
    CHECK(r.duty[0] == 300 && r.duty[1] == 0 && r.duty[2] == -300 && r.duty[3] == 1000);
    r.edge_bits = 0x8;
    sl_wheels(&r, w, now);
    run(&r, 5);
    CHECK(all_zero(&r));
}

int main(void) {
    test_crc();
    test_cobs_roundtrip();
    test_frame_roundtrip();
    test_stream_recovery();
    test_overflow();
    test_messages();
    test_no_motion_without_arm();
    test_watchdog();
    test_estop();
    test_edge_and_tof_blocking();
    test_battery();
    test_kinematics_and_ramp();
    test_wheels_mode();
    printf("%s: %d checks, %d failed\n", fails ? "FAILED" : "OK", checks, fails);
    return fails ? 1 : 0;
}
