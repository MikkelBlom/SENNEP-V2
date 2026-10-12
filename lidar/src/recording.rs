//! Recording from the UI. With a live LiDAR, everything is recorded automatically into
//! `<recordings>/auto/` (git-ignored, oldest files deleted). "Keep" moves the current recording out of
//! `auto/` under a name, where it becomes test data, and starts a new one. "Mark" notes a moment
//! ("bent down to fix the carpet") in a `.txt` next to the recording.
//!
//! Files are named by UTC start time; ~100 MB per hour of raw LiDAR bytes.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::source::Recorder;

/// A new auto file is started after this long, so no single file grows huge.
const ROTATE_S: f64 = 30.0 * 60.0;
/// Auto recordings kept (newest); older ones are deleted at start and on rotation.
const KEEP_AUTO: usize = 20;

struct Active {
    rec: Recorder,
    path: PathBuf,
    started: Instant,
    bytes: u64,
    marks: Vec<(f64, String)>,
}

pub struct Recordings {
    /// Where kept recordings go (`lidar/recordings` in the repo).
    pub keep_dir: PathBuf,
    pub auto_dir: PathBuf,
    /// False for replay/simulator sources: nothing to record.
    pub available: bool,
    cur: Option<Active>,
    last_kept: Option<String>,
    last_error: Option<String>,
}

impl Recordings {
    pub fn new(keep_dir: PathBuf, available: bool) -> Recordings {
        let auto_dir = keep_dir.join("auto");
        Recordings { keep_dir, auto_dir, available, cur: None, last_kept: None, last_error: None }
    }

    /// `lidar/recordings` when started from the repo root (where Mikkel runs the .exe),
    /// `recordings` when started inside `lidar/`.
    pub fn default_dir() -> PathBuf {
        for d in ["lidar/recordings", "recordings"] {
            if Path::new(d).is_dir() {
                return PathBuf::from(d);
            }
        }
        PathBuf::from("recordings")
    }

    pub fn start(&mut self) {
        if !self.available || self.cur.is_some() {
            return;
        }
        let r = fs::create_dir_all(&self.auto_dir).and_then(|_| {
            self.prune();
            let path = self.auto_dir.join(format!("{}.ldr", utc_stamp()));
            Recorder::create(&path).map(|rec| (rec, path))
        });
        match r {
            Ok((rec, path)) => {
                eprintln!("recording to {}", path.display());
                self.cur = Some(Active { rec, path, started: Instant::now(), bytes: 0, marks: Vec::new() });
                self.last_error = None;
            }
            Err(e) => self.fail(format!("cannot start recording: {e}")),
        }
    }

    /// Starts recording into an explicit file (the old `--record FILE` option).
    pub fn start_at(&mut self, path: &Path) {
        match Recorder::create(path) {
            Ok(rec) => {
                self.cur = Some(Active {
                    rec,
                    path: path.into(),
                    started: Instant::now(),
                    bytes: 0,
                    marks: Vec::new(),
                })
            }
            Err(e) => self.fail(format!("cannot create {}: {e}", path.display())),
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut a) = self.cur.take() {
            let _ = a.rec.flush();
            write_marks(&a);
        }
    }

    pub fn write(&mut self, host_us: u64, buf: &[u8]) {
        let Some(a) = &mut self.cur else { return };
        if let Err(e) = a.rec.write(host_us, buf) {
            self.cur = None;
            self.fail(format!("recording stopped: {e}"));
            return;
        }
        a.bytes += buf.len() as u64 + 12;
        if a.started.elapsed().as_secs_f64() > ROTATE_S && a.path.starts_with(&self.auto_dir) {
            self.stop();
            self.start();
        }
    }

    pub fn flush(&mut self) {
        if let Some(a) = &mut self.cur {
            let _ = a.rec.flush();
        }
    }

    pub fn mark(&mut self, text: &str) {
        if let Some(a) = &mut self.cur {
            let t = a.started.elapsed().as_secs_f64();
            let text = text.trim();
            a.marks.push((t, if text.is_empty() { "mark".into() } else { text.chars().take(200).collect() }));
            write_marks(a);
        }
    }

    /// Ends the current recording, moves it to `keep_dir/<name>.ldr` (+ its marks) and starts a new one.
    pub fn keep(&mut self, name: &str) {
        let Some(mut a) = self.cur.take() else { return };
        let _ = a.rec.flush();
        write_marks(&a);
        drop(a.rec);
        let base = sanitize(name);
        let mut target = self.keep_dir.join(format!("{base}.ldr"));
        let mut k = 2;
        while target.exists() {
            target = self.keep_dir.join(format!("{base}-{k}.ldr"));
            k += 1;
        }
        match fs::create_dir_all(&self.keep_dir).and_then(|_| fs::rename(&a.path, &target)) {
            Ok(()) => {
                let notes = a.path.with_extension("txt");
                if notes.exists() {
                    let _ = fs::rename(&notes, target.with_extension("txt"));
                }
                eprintln!("kept {}", target.display());
                self.last_kept = Some(target.display().to_string());
                self.last_error = None;
            }
            Err(e) => self.fail(format!("cannot keep recording: {e}")),
        }
        self.start();
    }

    /// Handles `rec:start`, `rec:stop`, `rec:keep <name>`, `rec:mark <text>` from the UI.
    pub fn command(&mut self, cmd: &str) {
        let (verb, arg) = cmd.split_once(' ').unwrap_or((cmd, ""));
        match verb {
            "rec:start" => self.start(),
            "rec:stop" => self.stop(),
            "rec:keep" => self.keep(arg),
            "rec:mark" => self.mark(arg),
            _ => {}
        }
    }

    pub fn status(&self) -> serde_json::Value {
        let cur = self.cur.as_ref().map(|a| {
            serde_json::json!({
                "file": a.path.file_name().map(|f| f.to_string_lossy().to_string()),
                "secs": a.started.elapsed().as_secs_f64(),
                "mb": a.bytes as f64 / 1e6,
                "marks": a.marks.len(),
                "auto": a.path.starts_with(&self.auto_dir),
            })
        });
        serde_json::json!({
            "available": self.available,
            "current": cur,
            "keep_dir": self.keep_dir.display().to_string(),
            "last_kept": self.last_kept,
            "error": self.last_error,
        })
    }

    fn fail(&mut self, msg: String) {
        eprintln!("{msg}");
        self.last_error = Some(msg);
    }

    fn prune(&self) {
        let Ok(rd) = fs::read_dir(&self.auto_dir) else { return };
        let mut files: Vec<PathBuf> =
            rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "ldr")).collect();
        files.sort(); // names are UTC timestamps: sorted = oldest first
        let n = files.len();
        for p in files.into_iter().take(n.saturating_sub(KEEP_AUTO - 1)) {
            let _ = fs::remove_file(&p);
            let _ = fs::remove_file(p.with_extension("txt"));
        }
    }
}

fn write_marks(a: &Active) {
    if a.marks.is_empty() {
        return;
    }
    let mut s = String::from("# Moments marked in the UI (time since the recording started)\n");
    for (t, text) in &a.marks {
        s += &format!("{:02}:{:04.1}  {}\n", (*t as u64) / 60, t % 60.0, text);
    }
    if let Err(e) = fs::write(a.path.with_extension("txt"), s) {
        eprintln!("cannot write marks: {e}");
    }
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(60)
        .collect();
    if s.trim_matches('_').is_empty() {
        format!("recording_{}", utc_stamp())
    } else {
        s
    }
}

/// `2026-10-12_22-47-03` (UTC), without a date crate.
fn utc_stamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Howard Hinnant's civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}_{:02}-{:02}-{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_moves_the_file_and_starts_a_new_one() {
        let dir = std::env::temp_dir().join(format!("sennep-rec-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut r = Recordings::new(dir.clone(), true);
        r.start();
        r.write(1, &[0x54, 0x2C, 1, 2, 3]);
        r.mark("bent down");
        r.keep("room 3!");
        let kept = dir.join("room_3_.ldr");
        assert!(kept.exists(), "{:?}", fs::read_dir(&dir).unwrap().collect::<Vec<_>>());
        assert!(fs::read_to_string(kept.with_extension("txt")).unwrap().contains("bent down"));
        assert!(r.cur.is_some(), "a new recording started");
        r.stop();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stamp_looks_like_a_date() {
        let s = utc_stamp();
        assert_eq!(s.len(), 19);
        assert!(s.starts_with("20"));
    }
}
