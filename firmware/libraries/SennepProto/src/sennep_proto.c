#include "sennep_proto.h"

uint16_t sp_crc16(const uint8_t *data, size_t len) {
    uint16_t crc = 0xFFFF;
    while (len--) {
        crc ^= (uint16_t)(*data++) << 8;
        for (uint8_t i = 0; i < 8; i++) crc = (crc & 0x8000) ? (uint16_t)((crc << 1) ^ 0x1021) : (uint16_t)(crc << 1);
    }
    return crc;
}

size_t sp_cobs_encode(const uint8_t *in, size_t len, uint8_t *out) {
    size_t o = 1, code_pos = 0;
    uint8_t code = 1;
    for (size_t i = 0; i < len; i++) {
        if (in[i] == 0) {
            out[code_pos] = code;
            code_pos = o++;
            code = 1;
        } else {
            out[o++] = in[i];
            if (++code == 0xFF) {
                out[code_pos] = code;
                code_pos = o++;
                code = 1;
            }
        }
    }
    out[code_pos] = code;
    return o;
}

size_t sp_cobs_decode(const uint8_t *in, size_t len, uint8_t *out) {
    size_t i = 0, o = 0;
    while (i < len) {
        uint8_t code = in[i++];
        if (code == 0) return 0;
        for (uint8_t k = 1; k < code; k++) {
            if (i >= len || in[i] == 0) return 0;
            out[o++] = in[i++];
        }
        if (code != 0xFF && i < len) out[o++] = 0;
    }
    return o;
}

size_t sp_frame_encode(uint8_t type, uint8_t seq, const uint8_t *payload, uint8_t len, uint8_t *out) {
    uint8_t raw[SP_MAX_RAW];
    if (len > SP_MAX_PAYLOAD) return 0;
    raw[0] = type;
    raw[1] = seq;
    sp_memcpy(raw + SP_HEADER, payload, len);
    uint16_t crc = sp_crc16(raw, (size_t)SP_HEADER + len);
    sp_put_u16(raw + SP_HEADER + len, crc);
    size_t n = sp_cobs_encode(raw, (size_t)SP_HEADER + len + SP_CRC, out);
    out[n++] = 0;
    return n;
}

static void reset_frame(sp_decoder_t *d) {
    d->n = 0;
    d->remaining = 0;
    d->code = 0;
    d->overflow = false;
}

void sp_decoder_init(sp_decoder_t *d) {
    memset(d, 0, sizeof(*d));
}

static bool push(sp_decoder_t *d, uint8_t b) {
    if (d->n >= SP_MAX_RAW) {
        d->overflow = true;
        return false;
    }
    d->buf[d->n++] = b;
    return true;
}

bool sp_decoder_feed(sp_decoder_t *d, uint8_t byte, sp_frame_t *out) {
    if (byte == 0) { /* delimiter: end of frame */
        bool started = d->code != 0 || d->overflow;
        bool complete = d->remaining == 0 && !d->overflow;
        uint8_t n = d->n;
        bool overflow = d->overflow;
        reset_frame(d);
        if (!started) return false; /* empty frame (e.g. back-to-back delimiters): ignore */
        if (!complete || overflow || n < SP_HEADER + SP_CRC) {
            d->frame_errors++;
            return false;
        }
        uint16_t crc = sp_get_u16(d->buf + n - SP_CRC);
        if (sp_crc16(d->buf, (size_t)n - SP_CRC) != crc) {
            d->crc_errors++;
            return false;
        }
        out->type = d->buf[0];
        out->seq = d->buf[1];
        out->len = (uint8_t)(n - SP_HEADER - SP_CRC);
        out->payload = d->buf + SP_HEADER;
        if (d->have_seq) d->seq_gaps = (uint16_t)(d->seq_gaps + (uint8_t)(out->seq - d->last_seq - 1u));
        d->have_seq = true;
        d->last_seq = out->seq;
        d->frames_ok++;
        return true;
    }
    if (d->overflow) return false;
    if (d->remaining == 0) { /* this byte is a COBS code byte */
        if (d->code != 0 && d->code != 0xFF) push(d, 0); /* previous block implied a zero */
        d->code = byte;
        d->remaining = (uint8_t)(byte - 1);
    } else {
        push(d, byte);
        d->remaining--;
    }
    return false;
}
