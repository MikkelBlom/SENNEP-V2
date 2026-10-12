// One COBS/CRC link on a hardware UART. Never blocks: frames that don't fit in the TX buffer are dropped and counted.
#pragma once
#include <Arduino.h>
#include <sennep_msgs.h>

class Link {
public:
    explicit Link(HardwareSerial &s) : serial_(s) {}
    void begin(unsigned long baud);

    // Reads everything waiting; returns true and fills `f` for each complete frame (call in a loop until false).
    bool poll(sp_frame_t &f);
    // Send a message payload. Returns false if dropped (TX buffer full).
    bool send(uint8_t type, const uint8_t *payload, uint8_t len);
    // Send an already-built raw frame (type|seq|payload|crc) unchanged, e.g. forwarded from the Pi to a node.
    bool sendRaw(const uint8_t *raw, uint8_t len);

    const sp_decoder_t &decoder() const { return dec_; }
    uint16_t txDropped() const { return tx_dropped_; }
    bool rxOverflowed() const { return rx_overflow_; }

private:
    bool write(const uint8_t *wire, size_t n);
    HardwareSerial &serial_;
    sp_decoder_t dec_;
    uint8_t tx_seq_ = 0;
    uint16_t tx_dropped_ = 0;
    bool rx_overflow_ = false;
};
