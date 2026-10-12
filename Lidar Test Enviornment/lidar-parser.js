const { Transform } = require('stream');

/**
 * LidarParser for LDROBOT LD19 / STL-19P Lidar
 * Protocol: 0x54 0x2C header, 47 bytes packet
 * Packet structure:
 * [0] Header: 0x54
 * [1] VerLen: 0x2C (12 points)
 * [2-3] Speed: Unit degrees/s
 * [4-5] Start Angle: Unit 0.01 deg
 * [6-41] Data: 12 points, 3 bytes each (Distance 2, Confidence 1)
 * [42-43] End Angle: Unit 0.01 deg
 * [44-45] Timestamp: Unit ms
 * [46] CRC8
 */
class LidarParser extends Transform {
  constructor() {
    super({ objectMode: true });
    this.buffer = Buffer.alloc(0);
    this.crcTable = this.generateCrcTable(0x4D);
  }

  /**
   * Generates CRC8 lookup table for polynomial 0x4D
   */
  generateCrcTable(poly) {
    const table = new Uint8Array(256);
    for (let i = 0; i < 256; i++) {
      let crc = 0;
      let byte = i;
      for (let j = 0; j < 8; j++) {
        if ((crc ^ byte) & 0x80) {
          crc = (crc << 1) ^ poly;
        } else {
          crc = (crc << 1);
        }
        byte = (byte << 1);
      }
      table[i] = crc & 0xFF;
    }
    return table;
  }

  /**
   * Calculates CRC8 for a buffer
   */
  calculateCrc(data) {
    let crc = 0;
    for (let i = 0; i < data.length; i++) {
      crc = this.crcTable[(crc ^ data[i]) & 0xFF];
    }
    return crc;
  }

  _transform(chunk, encoding, callback) {
    // Append new data to buffer
    this.buffer = Buffer.concat([this.buffer, chunk]);

    // Process all full packets in buffer
    while (this.buffer.length >= 47) {
      // Find packet header 0x54
      const headerIndex = this.buffer.indexOf(0x54);
      
      if (headerIndex === -1) {
        // No header found, clear buffer if it becomes too large
        if (this.buffer.length > 500) {
            this.buffer = Buffer.alloc(0);
        }
        break;
      }

      // Discard bytes before the header
      if (headerIndex > 0) {
        this.buffer = this.buffer.slice(headerIndex);
      }

      // Ensure we still have a full packet
      if (this.buffer.length < 47) break;

      // Check the VerLen byte (should be 0x2C)
      if (this.buffer[1] !== 0x2C) {
        // Not a valid packet start, discard this byte and search again
        this.buffer = this.buffer.slice(1);
        continue;
      }

      // Extract the 47-byte packet
      const packet = this.buffer.slice(0, 47);
      const receivedCrc = packet[46];
      const calculatedCrc = this.calculateCrc(packet.slice(0, 46));

      if (receivedCrc === calculatedCrc) {
        try {
          const parsedData = this.parsePacket(packet);
          this.push(parsedData);
          // Move buffer forward by one packet
          this.buffer = this.buffer.slice(47);
        } catch (err) {
          // Parsing error, discard this byte and continue
          this.buffer = this.buffer.slice(1);
        }
      } else {
        // Invalid checksum, discard the first byte and try to find header again
        this.buffer = this.buffer.slice(1);
      }
    }
    callback();
  }

  /**
   * Parses a single Lidar packet (47 bytes)
   */
  parsePacket(packet) {
    const speed = packet.readUInt16LE(2); // deg/s
    const startAngle = packet.readUInt16LE(4) / 100; // deg
    
    const points = [];
    for (let i = 0; i < 12; i++) {
      const offset = 6 + i * 3;
      const distance = packet.readUInt16LE(offset); // mm
      const confidence = packet[offset + 2];
      points.push({ distance, confidence });
    }

    const endAngle = packet.readUInt16LE(42) / 100; // deg
    const timestamp = packet.readUInt16LE(44); // ms

    // Calculate angular step for interpolation
    let diff = endAngle - startAngle;
    if (diff < 0) diff += 360;
    const step = diff / 11; // 12 points, 11 gaps

    return points.map((p, i) => ({
      angle: (startAngle + step * i) % 360,
      distance: p.distance,
      confidence: p.confidence,
      speed,
      timestamp
    }));
  }
}

module.exports = LidarParser;
