#include "link.h"

void Link::begin(unsigned long baud) {
    sp_decoder_init(&dec_);
    serial_.begin(baud);
    serial_.write((uint8_t)0); // flush any garbage the other side may have buffered before we booted
}

bool Link::poll(sp_frame_t &f) {
    // The core's RX buffer is 64 bytes: if it is full when we look, bytes may have been lost.
    int avail = serial_.available();
    if (avail >= SERIAL_RX_BUFFER_SIZE - 1) rx_overflow_ = true;
    while (avail-- > 0) {
        if (sp_decoder_feed(&dec_, (uint8_t)serial_.read(), &f)) return true;
    }
    return false;
}

bool Link::write(const uint8_t *wire, size_t n) {
    if ((size_t)serial_.availableForWrite() < n) {
        tx_dropped_++;
        return false;
    }
    serial_.write(wire, n);
    return true;
}

bool Link::send(uint8_t type, const uint8_t *payload, uint8_t len) {
    uint8_t wire[SP_MAX_WIRE];
    size_t n = sp_frame_encode(type, tx_seq_, payload, len, wire);
    if (n == 0) return false;
    if (!write(wire, n)) return false;
    tx_seq_++;
    return true;
}

bool Link::sendRaw(const uint8_t *raw, uint8_t len) {
    if (len > SP_MAX_RAW) return false;
    uint8_t wire[SP_MAX_WIRE];
    size_t n = sp_cobs_encode(raw, len, wire);
    wire[n++] = 0;
    return write(wire, n);
}
