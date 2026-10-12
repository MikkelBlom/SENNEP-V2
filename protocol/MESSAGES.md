# Beskeder — SENNEP Pi ↔ Mega

> Genereret fra `messages.json` af `tools/gen_msgs.py`. Protokolversion 1, spec-hash `0x5C25E06C`, maks payload 60 bytes. Little-endian.

Frame på ledningen: `COBS( type u8 | seq u8 | payload | crc16 u16 ) 0x00`, CRC-16/CCITT-FALSE over type+seq+payload.

| Id | Navn | Retning | Bytes | Felter | Beskrivelse |
|---|---|---|---:|---|---|
| 0x01 | `HEARTBEAT` | Pi → Mega | 0 | — | Keeps the link watchdog fed when no CMD_DRIVE is sent. |
| 0x02 | `ARM` | Pi → Mega | 0 | — | DISARMED -> ARMED. Rejected (ACK result) if e-stop/fault active. |
| 0x03 | `DISARM` | Pi → Mega | 0 | — | Any state -> DISARMED (also clears ESTOP_SW / LINK_LOST). |
| 0x04 | `ESTOP_SW` | Pi → Mega | 0 | — | Software e-stop. Latched until DISARM. |
| 0x10 | `CMD_DRIVE` | Pi → Mega | 7 | vx_mm_s `i16`, vy_mm_s `i16`, wz_mrad_s `i16`, flags `u8` | Body velocity. vx forward, vy left, wz counter-clockwise. Also feeds the watchdog. |
| 0x11 | `CMD_WHEELS` | Pi → Mega | 8 | duty_permille `i16[4]` | Direct wheel duty (test/calibration). Order FL, FR, RL, RR. -1000..1000. |
| 0x20 | `SET_PARAM` | Pi → Mega | 5 | param `u8`, value `f32` | Set a runtime parameter (RAM only). Answered by ACK. |
| 0x21 | `GET_PARAM` | Pi → Mega | 1 | param `u8` | Read a parameter. Answered by ACK with value. |
| 0x22 | `SET_OUTPUT` | Pi → Mega | 3 | output `u8`, value `u16` | Set a simple output (lights, buzzer). |
| 0x30 | `TIME_SYNC` | Pi → Mega | 4 | t_pi_us `u32` | Answered immediately by TIME_SYNC_REPLY. |
| 0x40 | `NODE_SEND` | Pi → Mega | 1+ | port `u8`, frame `u8[]` | Forward a raw (un-COBS'ed) frame to an expansion node port. |
| 0x80 | `HELLO` | Mega → Pi | 9 | proto_version `u8`, spec_hash `u32`, boot_reason `u8`, boot_count `u16`, max_payload `u8` | Sent at boot and at 2 Hz until the Pi talks. |
| 0x81 | `STATUS` | Mega → Pi | 20 | t_us `u32`, state `u8`, faults `u16`, blocked `u8`, battery_mv `u16`, current_ma `i16`, loop_max_us `u16`, rx_crc_errors `u16`, rx_frame_errors `u16`, rx_seq_gaps `u16` | 10 Hz health. |
| 0x82 | `IMU` | Mega → Pi | 18 | t_us `u32`, acc `i16[3]`, gyro `i16[3]`, temp `i16` | Raw MPU-6050 (default +-2 g, +-250 dps). |
| 0x83 | `MAG` | Mega → Pi | 10 | t_us `u32`, mag `i16[3]` | Raw QMC5883L. |
| 0x84 | `RANGES` | Mega → Pi | 29 | t_us `u32`, tof_mm `u16[8]`, sonar_mm `u16[4]`, edge_bits `u8` | 0 = no reading. tof_mm order counter-clockwise from front: F, FL, L, RL, B, RR, R, FR. sonar_mm: F, L, B, R. edge_bits: 1 = no floor, bit 0..3 = FL, FR, RL, RR corner. |
| 0x85 | `MOTORS` | Mega → Pi | 20 | t_us `u32`, duty_permille `i16[4]`, current_raw `u16[4]` | Applied duty after ramp/blocking, and raw BTS7960 IS readings. |
| 0x86 | `RC` | Mega → Pi | 12 | t_us `u32`, axis `i16[3]`, buttons `u8`, link_ok `u8` | RF joystick (NRF24) when active. |
| 0x87 | `TIME_SYNC_REPLY` | Mega → Pi | 8 | t_pi_us `u32`, t_mega_us `u32` | Echo of t_pi_us plus Mega micros() at reception. |
| 0x88 | `ACK` | Mega → Pi | 7 | msg_type `u8`, seq `u8`, result `u8`, value `f32` | Reply to ARM/DISARM/SET_PARAM/GET_PARAM/SET_OUTPUT/NODE_SEND. |
| 0x89 | `LOG` | Mega → Pi | 1+ | level `u8`, text `u8[]` | Short text. |
| 0xC0 | `NODE_FRAME` | Mega → Pi | 1+ | port `u8`, frame `u8[]` | A raw frame received from an expansion node, forwarded unchanged. |

## Konstanter

- **STATE:** BOOT = 0, DISARMED = 1, ARMED = 2, LINK_LOST = 3, ESTOP = 4, FAULT = 5
- **FAULT:** ESTOP_HW = 1, ESTOP_SW = 2, LINK_LOST = 4, BATTERY_LOW = 8, IMU_MISSING = 16, MAG_MISSING = 32, INA_MISSING = 64, LOOP_OVERRUN = 128, RX_OVERFLOW = 256
- **BLOCK:** FWD = 1, BACK = 2, LEFT = 4, RIGHT = 8, ROTATE = 16
- **BOOT_REASON:** UNKNOWN = 0, POWER_ON = 1, EXTERNAL = 2, BROWN_OUT = 3, WATCHDOG = 4
- **RESULT:** OK = 0, REJECTED = 1, BAD_PARAM = 2, BAD_LENGTH = 3, UNKNOWN_MSG = 4
- **PARAM:** LINK_TIMEOUT_MS = 0, RAMP_PERMILLE_PER_S = 1, STOP_RAMP_PERMILLE_PER_S = 2, MAX_WHEEL_MM_S = 3, ROT_ARM_MM = 4, TOF_STOP_MM = 5, EDGE_ENABLE = 6, BATTERY_LOW_MV = 7, MOTOR_INVERT_MASK = 8
- **OUTPUT:** HEADLIGHTS = 0, TAILLIGHTS = 1, BUZZER = 2
- **PORT:** NODE2 = 2, NODE3 = 3
- **DRIVE_FLAG:** NONE = 0
