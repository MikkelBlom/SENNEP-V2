# sennep-lidar — LiDAR stack v2 (Rust)

New LiDAR processing for SENNEP V2's LDROBOT STL-19P (LD19 protocol). Replaces the old
Electron app in `Lidar Test Enviornment/` (which is only used as test data). Plan and reasons:
[`docs/lidar/PLAN.md`](../docs/lidar/PLAN.md). Measured numbers: [`docs/lidar/BENCHMARK.md`](../docs/lidar/BENCHMARK.md).

**What it does:** reads the LiDAR (the USB adapter), computes safety zones every 2.4 ms, assembles whole
revolutions, finds the robot's movement (LiDAR odometry), builds a map, draws **walls as straight lines** and **objects as
outlines**, **tracks moving things** (position, speed, trail, where they will be in 1 s) and shows it all on a
web page that you open from a PC, phone or the robot's 7" screen. One program, one file, ~6 MB RAM.

## Run it on your PC (without hardware)

```bash
cd lidar
cargo run --release -- --sim --person                 # simulated flat with a walking person
cargo run --release -- --replay recordings/2026-10-12_room_walkthrough.ldr --loop --no-body
cargo run --release -- --replay "../Lidar Test Enviornment/recordings/<file>.jsonl" --loop --no-body
```
Real LiDAR on a PC (Windows: the COM port from Device Manager), started from the repo root:
`sennep-lidar --serial COM5 --no-body --http 127.0.0.1:8080`.
**It records by itself** into `lidar/recordings/auto/` (git-ignored, newest 20 files kept, a new file every 30 min).
The panel bottom-left on the page: **Mark moment** (a note with the time, saved next to the recording),
**Keep recording…** (moves it to `lidar/recordings/<name>.ldr` + `.txt`, where it becomes test data, and starts a
new one), **Stop / Start recording**. Options: `--no-record`, `--record-dir DIR`, `--record FILE.ldr`.

**Playback:** the **Recordings** button lists kept and automatic recordings; **Play** opens one in the same page
(the live LiDAR pauses, recording stops). Player bar: play/pause (space), ±5 s (arrow keys), type a time like
`1:32` + Enter, drag the timeline, speed 0.25–4×, marks as ticks/buttons (click = jump to 2 s before),
**Back to live**. Jumping rebuilds the map from the start of the recording, ~1 s per minute.
`sennep-lidar --replay <file.ldr>` opens straight into playback.

**Layers** (buttons on the page): Walls · Objects · Movement · Points · Map pixels. Walls are fitted to the
map (which averages hundreds of revolutions), keep an id and only move when the map does; their ends glide on
screen. Moving things are red: a ring, a trail (4 s) and a dashed arrow to where they will be in 1 s. "(lost)" =
not seen right now; the position is a guess from the last speed for up to 2 s.
Open http://localhost:8080. `--no-body` is used when the LiDAR is not mounted on the robot (the old recordings
were made on a desk); otherwise everything inside the robot's outline (800 × 650 mm) is ignored.

## Run it on the Pi

1. Build on the PC (no cross tools needed; gives one static file):
   ```bash
   rustup target add aarch64-unknown-linux-musl
   cargo build --release --target aarch64-unknown-linux-musl
   scp target/aarch64-unknown-linux-musl/release/{sennep-lidar,bench} sennep@<pi>:~
   ```
   (Or build directly on the Pi with `cargo build --release`; it takes a few minutes.)
2. Find the adapter's fixed name: `ls /dev/serial/by-id/` (the name does not change, as `/dev/ttyUSB0` can).
   The user must be in `dialout`: `sudo usermod -aG dialout sennep` (log out and in).
3. Try: `./sennep-lidar --serial /dev/serial/by-id/usb-Silicon_Labs_…`, and open `http://<pi>:8080`.
4. Record for later tests: add `--record tur1.ldr` (raw bytes + time; replayed bit for bit with `--replay tur1.ldr`).
5. Start automatically: `deploy/sennep-lidar.service` (instructions at the top of the file).
6. **Measure the Pi:** `./bench --long --recordings "<repo>/Lidar Test Enviornment/recordings" --out BENCHMARK-pi.md`
   and put the result in `docs/lidar/`.

**7" screen:** open the page in Chromium in kiosk mode, e.g. `chromium-browser --kiosk http://localhost:8080`.
How to autostart it depends on the Pi OS version (Wayland/labwc in newer versions), so it is not
set up yet. The page only draws when new data arrives, so it does not load the Pi much.

## Test

```bash
cargo test --release          # 36 tests: parser, clock, filters, zones, walls, motion, recordings, simulator
cargo run --release --bin bench -- --long
```
The tests run the old recordings through the real parser, and the simulator (known ground truth) through
the whole chain. The simulator makes **raw LD19 bytes**, incl. bit errors on the wire and a person walking around.

## Structure

| File | What |
|---|---|
| `protocol.rs` | LD19 packets: resync, CRC8, angles (tested against the manual's example) |
| `source.rs` | `ByteSource`: serial, replay (`.ldr` and old `.jsonl`), simulator |
| `clock.rs` | The LiDAR's ms clock (wrap 30 s) → the Pi's clock |
| `scan.rs` | Whole revolutions, split exactly at 360°→0°, robot frame, time per point |
| `filter.rs` | Distances, the robot's own body, LDROBOT-like noise filter (only for odometry/map) |
| `zones.rs` | 16 sectors: free space from the robot's edge, updated per packet, "too old" flag |
| `odom.rs` | Prediction → de-skew → point-to-line ICP against nearby keyframes |
| `grid.rs` | Log-odds map (5 cm), free space cleared by rays, changed tiles are sent |
| `walls.rs` | Wall segments fitted to the map (re-fit known walls first, RANSAC for new ones, sticky corners), objects = the rest |
| `recording.rs` | Auto-recording, keep/mark/stop/start from the page |
| `player.rs` | Playback in the page: pause, speed, jump (map rebuilt from the start), marks |
| `motion.rs` | Moving points (hit in known-free space, ≥ 10 cm from anything solid), blobs, Kalman tracks with coasting |
| `server.rs`, `web/index.html` | HTTP + WebSocket, one static page (no build step) |
| `sim.rs`, `eval.rs` | Simulated flat, trips, ground truth and scoring |

Coordinates: x forward, y left, angles counter-clockwise (ROS convention). The LiDAR's 0° = the robot's front
(`Mount` in `scan.rs`, if it is mounted differently).

## For the Mega

`ZoneReport::encode()` gives 41 bytes (time, stale mask, 16 × free space in mm) — within the link's
60-byte payload (`docs/electronics/PI_MEGA_LINK.md`). Sector 0 is straight ahead, the sectors go counter-clockwise.
**The Mega must treat a sector marked "stale" as blocked** (the LiDAR has stalled).
The actual sending over the link is done when the link is built (the `ObstacleSink` trait is ready for it).

## Known limitations (honestly)

- **Not tested on the real LiDAR or the Pi yet** — only recordings, simulator and ARM emulation.
- The zones show the latest pass, i.e. up to **100 ms old** (one revolution). At 0.4 m/s that is 4 cm.
- **Thin things far away** (a 4 cm chair leg at 2–4 m) are not hit by every revolution; the sample spacing is
  0.72°. The ToF sensors on the Mega cover the near area.
- The LiDAR only sees at one height (~285 mm above the floor). Low things (plinths, cables) are not seen.
- No loop closure yet: the keyframe map holds the position in places the robot has been, but a
  very large loop (other rooms, long corridors) can still give drift. The gyro (MPU-6050) is not fused in yet.
- The map is not saved between restarts yet.
- **Movement needs a known map:** something moving in a spot the LiDAR has never seen empty is not flagged
  (it takes ~1 s of view). Something that stands still long enough becomes part of the map. A thing must move
  ≥ 25 cm before it is shown (that removes map artefacts at door frames while the robot drives).
- Moving things within 10 cm of a wall are not flagged. Two people walking side by side (< 35 cm) are one track.
- A "lost" track slows down and stays within 1 m of where it was last seen, for up to 3 s. Something that
  reappears within 1.2 m of it gets the same id (bending down below the scan plane, a sprinter turning at a wall).
  Two people crossing close to each other can swap ids. A hidden person is remembered for 10 s.
- Small things (< 6 points on average) never become movers: a person far away (> ~4 m) with only a few points is
  not shown until they come closer. Spots that keep flickering (glass: a kettle, a cabinet door) are learned
  and ignored for small blobs. A blob within 70 cm of a tracked person is treated as part of them.
