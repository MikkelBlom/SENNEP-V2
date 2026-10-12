/*
 * SENNEP Pi <-> Mega framing: COBS + CRC-16/CCITT-FALSE.
 * Portable C99, no Arduino dependency: the same file runs on the Mega, on a PC (tests) and on the Pi.
 *
 * Raw frame:   type u8 | seq u8 | payload (0..SP_MAX_PAYLOAD) | crc16 u16 LE   (crc over type+seq+payload)
 * On the wire: COBS(raw frame) followed by a single 0x00 delimiter.
 */
#ifndef SENNEP_PROTO_H
#define SENNEP_PROTO_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SP_MAX_PAYLOAD 60u
#define SP_HEADER 2u                                   /* type + seq */
#define SP_CRC 2u
#define SP_MAX_RAW (SP_HEADER + SP_MAX_PAYLOAD + SP_CRC) /* 64 */
#define SP_MAX_WIRE (SP_MAX_RAW + SP_MAX_RAW / 254u + 2u) /* COBS overhead + delimiter = 66 */

/* ---- little-endian helpers (used by the generated sennep_msgs.h) ---- */
static inline void sp_memcpy(uint8_t *d, const uint8_t *s, uint8_t n) { if (n) memcpy(d, s, n); }
static inline void sp_put_u8(uint8_t *p, uint8_t v) { p[0] = v; }
static inline void sp_put_i8(uint8_t *p, int8_t v) { p[0] = (uint8_t)v; }
static inline void sp_put_u16(uint8_t *p, uint16_t v) { p[0] = (uint8_t)v; p[1] = (uint8_t)(v >> 8); }
static inline void sp_put_i16(uint8_t *p, int16_t v) { sp_put_u16(p, (uint16_t)v); }
static inline void sp_put_u32(uint8_t *p, uint32_t v) {
    p[0] = (uint8_t)v; p[1] = (uint8_t)(v >> 8); p[2] = (uint8_t)(v >> 16); p[3] = (uint8_t)(v >> 24);
}
static inline void sp_put_i32(uint8_t *p, int32_t v) { sp_put_u32(p, (uint32_t)v); }
static inline void sp_put_f32(uint8_t *p, float v) { uint32_t u; memcpy(&u, &v, 4); sp_put_u32(p, u); }
static inline uint8_t sp_get_u8(const uint8_t *p) { return p[0]; }
static inline int8_t sp_get_i8(const uint8_t *p) { return (int8_t)p[0]; }
static inline uint16_t sp_get_u16(const uint8_t *p) { return (uint16_t)(p[0] | ((uint16_t)p[1] << 8)); }
static inline int16_t sp_get_i16(const uint8_t *p) { return (int16_t)sp_get_u16(p); }
static inline uint32_t sp_get_u32(const uint8_t *p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}
static inline int32_t sp_get_i32(const uint8_t *p) { return (int32_t)sp_get_u32(p); }
static inline float sp_get_f32(const uint8_t *p) { uint32_t u = sp_get_u32(p); float v; memcpy(&v, &u, 4); return v; }

/* ---- primitives ---- */
uint16_t sp_crc16(const uint8_t *data, size_t len);                 /* CRC-16/CCITT-FALSE, "123456789" -> 0x29B1 */
size_t sp_cobs_encode(const uint8_t *in, size_t len, uint8_t *out); /* no delimiter appended; out >= len + len/254 + 1 */
size_t sp_cobs_decode(const uint8_t *in, size_t len, uint8_t *out); /* returns 0 on malformed input */

/* Build a complete wire frame (COBS + trailing 0x00) into out[SP_MAX_WIRE]. Returns bytes to send, 0 if payload too long. */
size_t sp_frame_encode(uint8_t type, uint8_t seq, const uint8_t *payload, uint8_t len, uint8_t *out);

/* A decoded, CRC-checked frame. payload points into the decoder's buffer: valid until the next sp_decoder_feed(). */
typedef struct {
    uint8_t type;
    uint8_t seq;
    uint8_t len;
    const uint8_t *payload;
} sp_frame_t;

/* Streaming decoder: feed one byte at a time; COBS is undone on the fly (64 bytes of RAM per port). */
typedef struct {
    uint8_t buf[SP_MAX_RAW];
    uint8_t n;          /* decoded bytes so far */
    uint8_t remaining;  /* data bytes left in the current COBS block */
    uint8_t code;       /* code byte of the current block, 0 = none yet */
    bool overflow;      /* frame too long: drop until next delimiter */
    bool have_seq;
    uint8_t last_seq;
    /* counters (wrap) */
    uint16_t frames_ok;
    uint16_t crc_errors;
    uint16_t frame_errors; /* malformed COBS, too short, too long */
    uint16_t seq_gaps;     /* frames missing according to seq */
} sp_decoder_t;

void sp_decoder_init(sp_decoder_t *d);
/* Returns true when `out` holds a new valid frame. */
bool sp_decoder_feed(sp_decoder_t *d, uint8_t byte, sp_frame_t *out);

#ifdef __cplusplus
}
#endif
#endif
