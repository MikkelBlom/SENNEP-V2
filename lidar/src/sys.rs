//! Own-process CPU time and memory, from /proc (Linux, i.e. the Pi). Other OSes report None.

use std::fs;

/// Total user + system CPU time of this process, seconds.
pub fn cpu_seconds() -> Option<f64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    // Fields after the parenthesised command name; utime and stime are fields 14 and 15.
    let rest = &stat[stat.rfind(')')? + 2..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    let utime: f64 = f.get(11)?.parse().ok()?;
    let stime: f64 = f.get(12)?.parse().ok()?;
    Some((utime + stime) / clock_ticks())
}

fn clock_ticks() -> f64 {
    // USER_HZ is 100 on every Linux the Pi runs; avoid a libc dependency for sysconf.
    100.0
}

/// Resident set size (current, peak) in MB.
pub fn rss_mb() -> Option<(f64, f64)> {
    let s = fs::read_to_string("/proc/self/status").ok()?;
    let kb = |key: &str| -> Option<f64> {
        s.lines().find(|l| l.starts_with(key))?.split_whitespace().nth(1)?.parse::<f64>().ok()
    };
    Some((kb("VmRSS:")? / 1024.0, kb("VmHWM:")? / 1024.0))
}

/// Tracks CPU usage between calls as a percentage of one core.
pub struct CpuMeter {
    last: Option<(f64, std::time::Instant)>,
}

impl Default for CpuMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuMeter {
    pub fn new() -> Self {
        CpuMeter { last: cpu_seconds().map(|c| (c, std::time::Instant::now())) }
    }
    pub fn sample(&mut self) -> Option<f64> {
        let now = (cpu_seconds()?, std::time::Instant::now());
        let pct =
            self.last.map(|(c, t)| (now.0 - c) / now.1.duration_since(t).as_secs_f64().max(1e-3) * 100.0);
        self.last = Some(now);
        pct
    }
}
