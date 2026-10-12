# LiDAR recordings (test data)

Raw LD19 bytes with timestamps (`.ldr`). Replay: `sennep-lidar --replay <file> --loop --no-body`.
Each has a `.txt` with what happens when. The tests in `lidar/tests/` replay them, so keep them small and meaningful.

New recordings: the program records a live LiDAR by itself into `auto/` (git-ignored, the newest 20 files are
kept). Press **Keep recording…** on the page to move the current one here under a name; **Mark moment** writes
a line into its `.txt`. Commit the kept ones you want as test data.
