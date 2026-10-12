//! Playback of recordings in the UI: play / pause / speed / jump to a time, with the moments marked
//! while recording shown on the timeline. Jumping rebuilds the map by running the recording from the
//! start as fast as possible up to that time (the pipeline is deterministic, so it is exactly what
//! was on screen live; ~1 s per minute of recording).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::source::load_ldr;

pub struct Player {
    pub path: PathBuf,
    chunks: Vec<(u64, Vec<u8>)>,
    next: usize,
    pub paused: bool,
    pub speed: f64,
    pub looped: bool,
    /// Wall clock and recording time (µs since the first chunk) at the last (re)start.
    anchor: (Instant, u64),
    pub marks: Vec<(f64, String)>,
    /// Added to host times after a loop so time keeps increasing for the pipeline.
    shift_us: u64,
}

impl Player {
    pub fn open(path: &Path, looped: bool) -> io::Result<Player> {
        let chunks = load_ldr(path)?;
        if chunks.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "empty recording"));
        }
        Ok(Player {
            path: path.to_path_buf(),
            chunks,
            next: 0,
            paused: false,
            speed: 1.0,
            looped,
            anchor: (Instant::now(), 0),
            marks: read_marks(path),
            shift_us: 0,
        })
    }

    fn rel(&self, i: usize) -> u64 {
        self.chunks[i].0 - self.chunks[0].0
    }

    pub fn duration_s(&self) -> f64 {
        self.rel(self.chunks.len() - 1) as f64 * 1e-6
    }

    /// Recording time of the last chunk handed out, s.
    pub fn pos_s(&self) -> f64 {
        if self.next == 0 {
            0.0
        } else {
            self.rel(self.next - 1) as f64 * 1e-6
        }
    }

    pub fn at_end(&self) -> bool {
        self.next >= self.chunks.len()
    }

    fn reanchor(&mut self) {
        let rec = if self.next == 0 { 0 } else { self.rel(self.next - 1) };
        self.anchor = (Instant::now(), rec);
    }

    pub fn set_paused(&mut self, p: bool) {
        if !p && self.at_end() {
            self.next = 0; // play again from the start (the caller resets the pipeline)
        }
        self.paused = p;
        self.reanchor();
    }

    pub fn set_speed(&mut self, s: f64) {
        self.speed = s.clamp(0.1, 16.0);
        self.reanchor();
    }

    /// The next chunk if it is due (paced by the recording's own timestamps × speed). `None` = nothing
    /// due yet (waits up to 10 ms), paused, or at the end.
    pub fn poll(&mut self, buf: &mut Vec<u8>) -> Option<u64> {
        if self.paused {
            std::thread::sleep(Duration::from_millis(10));
            return None;
        }
        if self.at_end() {
            if self.looped {
                self.shift_us += self.chunks.last().unwrap().0 - self.chunks[0].0 + 100_000;
                self.next = 0;
                self.reanchor();
            } else {
                self.paused = true;
                return None;
            }
        }
        let due_rec = self.rel(self.next);
        let wall = self.anchor.0.elapsed().as_secs_f64() * self.speed;
        let due = (due_rec.saturating_sub(self.anchor.1)) as f64 * 1e-6;
        if due > wall {
            let wait = ((due - wall) / self.speed).min(0.01);
            std::thread::sleep(Duration::from_secs_f64(wait));
            if due > self.anchor.0.elapsed().as_secs_f64() * self.speed {
                return None;
            }
        }
        let (t, bytes) = &self.chunks[self.next];
        self.next += 1;
        buf.clear();
        buf.extend_from_slice(bytes);
        Some(t + self.shift_us)
    }

    /// Chunks from the start up to `t_s`, for rebuilding the map before a jump. Afterwards playback
    /// continues from `t_s`.
    pub fn seek(&mut self, t_s: f64, mut feed: impl FnMut(&[u8], u64)) {
        let target = (t_s.clamp(0.0, self.duration_s()) * 1e6) as u64;
        self.shift_us = 0;
        let mut i = 0;
        while i < self.chunks.len() && self.rel(i) < target {
            feed(&self.chunks[i].1, self.chunks[i].0);
            i += 1;
        }
        self.next = i;
        self.reanchor();
    }
}

/// `mm:ss.s  text` lines from the recording's `.txt` (written by "Mark moment").
pub fn read_marks(path: &Path) -> Vec<(f64, String)> {
    let Ok(s) = fs::read_to_string(path.with_extension("txt")) else { return Vec::new() };
    s.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| {
            let (time, text) = l.trim().split_once(char::is_whitespace)?;
            let (m, sec) = time.split_once(':')?;
            Some((m.parse::<f64>().ok()? * 60.0 + sec.parse::<f64>().ok()?, text.trim().to_string()))
        })
        .collect()
}

/// Recordings the UI can play: kept ones first (newest first), then the automatic ones.
pub fn list(keep_dir: &Path) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for (dir, kept) in [(keep_dir.to_path_buf(), true), (keep_dir.join("auto"), false)] {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        let mut files: Vec<(std::time::SystemTime, PathBuf, u64)> = rd
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "ldr"))
            .filter_map(|e| {
                let m = e.metadata().ok()?;
                Some((m.modified().ok()?, e.path(), m.len()))
            })
            .collect();
        files.sort_by_key(|f| std::cmp::Reverse(f.0));
        for (_, p, len) in files {
            out.push(serde_json::json!({
                "path": p.display().to_string(),
                "name": p.file_stem().map(|s| s.to_string_lossy().to_string()),
                "kept": kept,
                "mb": len as f64 / 1e6,
                // ~27 kB per second of LiDAR bytes + chunk headers
                "secs": len as f64 / 27_500.0,
                "marks": read_marks(&p).len(),
            }));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_are_read_back() {
        let p = std::env::temp_dir().join(format!("sennep-marks-{}.ldr", std::process::id()));
        fs::write(
            p.with_extension("txt"),
            "# header\n01:30.6  bends down to fix the carpet\n00:05.0  walks in\n",
        )
        .unwrap();
        let m = read_marks(&p);
        assert_eq!(m.len(), 2);
        assert!((m[0].0 - 90.6).abs() < 1e-9 && m[0].1 == "bends down to fix the carpet");
        let _ = fs::remove_file(p.with_extension("txt"));
    }
}
