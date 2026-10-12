//! Tiny HTTP + WebSocket server for the UI. Serves one static page and streams binary frames.
//!
//! Each viewer gets its own thread that always sends the *newest* data and skips anything it
//! missed. A slow phone on WiFi can therefore never build a queue or slow down the robot, which
//! was one source of the old app's stutter (every frame was pushed through IPC as JSON).
//!
//! Frames (little endian), first byte = type:
//! 1 scan:  u8 1, u8 accepted, u16 n, u32 seq, f64 t, f32 x, f32 y, f32 th, n x (i16 x_mm, i16 y_mm) robot frame
//! 2 zones: u8 2, u8 n, u16 0, u32 stale mask, f64 t, n x u16 clearance_mm
//! 3 tiles: u8 3, u8 0, u16 count, f32 res, f32 origin_x, f32 origin_y, u16 tile, u16 tiles_per_side,
//!          count x (u16 tx, u16 ty, tile*tile i8 log-odds)
//! 4 walls: u8 4, u8 0, u16 walls, u16 objects, u16 0,
//!          walls x (u32 id, f32 ax, f32 ay, f32 bx, f32 by), objects x (u16 n, n x (f32 x, f32 y)) world frame
//! 5 motion: u8 5, u8 0, u16 tracks, u16 points, u16 0, f64 t,
//!          tracks x (u32 id, u8 flags (1 = coasting), u8 0, u16 points, f32 x, y, vx, vy), points x (f32 x, f32 y)
//! Text frames carry JSON diagnostics once per second.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use tungstenite::Message;

use crate::grid::{Grid, TILE};
use crate::motion::Tracker;
use crate::pipeline::ScanResult;
use crate::walls::WallMap;
use crate::zones::ZoneReport;

pub const INDEX_HTML: &str = include_str!("../web/index.html");

#[derive(Default)]
struct State {
    version: u64,
    scan: Option<Arc<Vec<u8>>>,
    scan_version: u64,
    zones: Option<Arc<Vec<u8>>>,
    zones_version: u64,
    stats: Option<Arc<String>>,
    stats_version: u64,
    walls: Option<Arc<Vec<u8>>>,
    walls_version: u64,
    motion: Option<Arc<Vec<u8>>>,
    motion_version: u64,
    /// Encoded tile payloads and the version at which each last changed.
    tiles: Vec<Option<Arc<Vec<u8>>>>,
    tile_version: Vec<u64>,
    grid_header: Option<(f32, f32, f32, u16, u16)>,
    /// Bumped on map reset so viewers drop their old map.
    epoch: u64,
}

/// Shared between the processing thread (publisher) and viewer threads.
pub struct Hub {
    state: Mutex<State>,
    cond: Condvar,
    pub reset_requested: AtomicBool,
    pub clients: AtomicUsize,
    /// Text commands from the page for the main loop (recording control).
    pub commands: Mutex<Vec<String>>,
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            state: Mutex::new(State::default()),
            cond: Condvar::new(),
            reset_requested: AtomicBool::new(false),
            clients: AtomicUsize::new(0),
            commands: Mutex::new(Vec::new()),
        }
    }

    fn bump(&self, f: impl FnOnce(&mut State, u64)) {
        let mut s = self.state.lock().unwrap();
        s.version += 1;
        let v = s.version;
        f(&mut s, v);
        drop(s);
        self.cond.notify_all();
    }

    pub fn publish_scan(&self, r: &ScanResult) {
        let mut b = Vec::with_capacity(28 + r.raw.len() * 4);
        b.push(1);
        b.push(r.info.accepted as u8);
        b.extend_from_slice(&(r.raw.len().min(u16::MAX as usize) as u16).to_le_bytes());
        b.extend_from_slice(&(r.seq as u32).to_le_bytes());
        b.extend_from_slice(&r.t_end.to_le_bytes());
        for v in [r.pose.x as f32, r.pose.y as f32, r.pose.th as f32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for p in r.raw.iter().take(u16::MAX as usize) {
            let q = |v: f32| ((v * 1000.0).round().clamp(-32_767.0, 32_767.0) as i16).to_le_bytes();
            b.extend_from_slice(&q(p.x));
            b.extend_from_slice(&q(p.y));
        }
        self.bump(|s, v| {
            s.scan = Some(Arc::new(b));
            s.scan_version = v;
        });
    }

    pub fn publish_zones(&self, z: &ZoneReport) {
        let mut b = Vec::with_capacity(16 + z.clearance.len() * 2);
        b.push(2);
        b.push(z.clearance.len() as u8);
        b.extend_from_slice(&[0, 0]);
        let mask = z.stale.iter().enumerate().fold(0u32, |m, (i, &s)| m | ((s as u32) << i));
        b.extend_from_slice(&mask.to_le_bytes());
        b.extend_from_slice(&z.t.to_le_bytes());
        for &c in &z.clearance {
            b.extend_from_slice(&((c * 1000.0).round().clamp(0.0, 65_535.0) as u16).to_le_bytes());
        }
        self.bump(|s, v| {
            s.zones = Some(Arc::new(b));
            s.zones_version = v;
        });
    }

    pub fn publish_stats(&self, json: String) {
        self.bump(|s, v| {
            s.stats = Some(Arc::new(json));
            s.stats_version = v;
        });
    }

    pub fn publish_walls(&self, wm: &WallMap) {
        let walls = wm.output();
        let mut b = Vec::with_capacity(8 + walls.len() * 20 + wm.objects.len() * 40);
        b.extend_from_slice(&[4, 0]);
        b.extend_from_slice(&(walls.len() as u16).to_le_bytes());
        b.extend_from_slice(&(wm.objects.len() as u16).to_le_bytes());
        b.extend_from_slice(&[0, 0]);
        for w in &walls {
            b.extend_from_slice(&w.id.to_le_bytes());
            for v in [w.a.0, w.a.1, w.b.0, w.b.1] {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        for o in &wm.objects {
            b.extend_from_slice(&(o.hull.len() as u16).to_le_bytes());
            for &(x, y) in &o.hull {
                b.extend_from_slice(&x.to_le_bytes());
                b.extend_from_slice(&y.to_le_bytes());
            }
        }
        self.bump(|s, v| {
            s.walls = Some(Arc::new(b));
            s.walls_version = v;
        });
    }

    /// Confirmed tracks + this scan's moving points.
    pub fn publish_motion(&self, t: f64, tracker: &Tracker, moving: &[(f32, f32)]) {
        let tracks: Vec<_> = tracker.tracks.iter().filter(|k| k.confirmed).collect();
        let mut b = Vec::with_capacity(16 + tracks.len() * 24 + moving.len() * 8);
        b.extend_from_slice(&[5, 0]);
        b.extend_from_slice(&(tracks.len() as u16).to_le_bytes());
        b.extend_from_slice(&(moving.len().min(u16::MAX as usize) as u16).to_le_bytes());
        b.extend_from_slice(&[0, 0]);
        b.extend_from_slice(&t.to_le_bytes());
        for k in tracks {
            b.extend_from_slice(&k.id.to_le_bytes());
            b.push(k.coasting() as u8);
            b.push(0);
            b.extend_from_slice(&(k.points.min(u16::MAX as usize) as u16).to_le_bytes());
            let ((x, y), (vx, vy)) = (k.pos(), k.vel());
            for v in [x, y, vx, vy] {
                b.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for &(x, y) in moving.iter().take(u16::MAX as usize) {
            b.extend_from_slice(&x.to_le_bytes());
            b.extend_from_slice(&y.to_le_bytes());
        }
        self.bump(|s, v| {
            s.motion = Some(Arc::new(b));
            s.motion_version = v;
        });
    }

    /// Copies the grid's changed tiles. Call after each integrated scan.
    pub fn publish_grid(&self, g: &mut Grid) {
        let dirty = g.take_dirty();
        if dirty.is_empty() {
            return;
        }
        let tps = g.tiles_per_side();
        let encoded: Vec<(usize, Vec<u8>)> = dirty
            .into_iter()
            .map(|(tx, ty)| {
                let mut b = Vec::with_capacity(4 + TILE * TILE);
                b.extend_from_slice(&(tx as u16).to_le_bytes());
                b.extend_from_slice(&(ty as u16).to_le_bytes());
                g.tile(tx, ty, &mut b);
                (ty * tps + tx, b)
            })
            .collect();
        let header = (g.cfg.resolution, g.origin.0, g.origin.1, TILE as u16, tps as u16);
        self.bump(|s, v| {
            if s.tiles.len() != tps * tps {
                s.tiles = vec![None; tps * tps];
                s.tile_version = vec![0; tps * tps];
            }
            s.grid_header = Some(header);
            for (i, b) in encoded {
                s.tiles[i] = Some(Arc::new(b));
                s.tile_version[i] = v;
            }
        });
    }

    pub fn reset_map(&self) {
        self.bump(|s, _| {
            s.tiles.clear();
            s.tile_version.clear();
            s.walls = None;
            s.motion = None;
            s.epoch += 1;
        });
    }
}

pub fn serve(addr: &str, hub: Arc<Hub>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    eprintln!("UI: http://{addr}/  (on the Pi: http://<pi-address>:{})", listener.local_addr()?.port());
    std::thread::Builder::new().name("http".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let hub = hub.clone();
            let _ = std::thread::Builder::new().name("viewer".into()).spawn(move || {
                let _ = handle(stream, hub);
            });
        }
    })?;
    Ok(())
}

fn handle(stream: TcpStream, hub: Arc<Hub>) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut peek = [0u8; 512];
    let n = stream.peek(&mut peek)?;
    let head = String::from_utf8_lossy(&peek[..n]);
    let path = head.split_whitespace().nth(1).unwrap_or("/").to_string();
    if path.starts_with("/ws") {
        return viewer(stream, hub);
    }
    let mut stream = stream;
    let mut req = [0u8; 4096];
    let _ = stream.read(&mut req)?;
    let (status, ctype, body) = match path.as_str() {
        "/" | "/index.html" => ("200 OK", "text/html; charset=utf-8", INDEX_HTML.as_bytes()),
        _ => ("404 Not Found", "text/plain", b"not found".as_slice()),
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

fn viewer(stream: TcpStream, hub: Arc<Hub>) -> io::Result<()> {
    stream.set_nodelay(true)?;
    let mut ws = tungstenite::accept(stream).map_err(|e| io::Error::other(e.to_string()))?;
    ws.get_mut().set_read_timeout(Some(Duration::from_millis(5)))?;
    hub.clients.fetch_add(1, Ordering::Relaxed);
    let result = viewer_loop(&mut ws, &hub);
    hub.clients.fetch_sub(1, Ordering::Relaxed);
    result
}

fn viewer_loop(ws: &mut tungstenite::WebSocket<TcpStream>, hub: &Hub) -> io::Result<()> {
    let err = |e: tungstenite::Error| io::Error::other(e.to_string());
    let (mut seen, mut seen_scan, mut seen_zones, mut seen_stats, mut seen_tiles) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut seen_walls, mut seen_motion) = (0u64, 0u64);
    let mut epoch = u64::MAX;
    loop {
        // Wait for anything new, then take only the newest of each kind.
        let (scan, zones, stats, tiles, walls, motion) = {
            let mut s = hub.state.lock().unwrap();
            while s.version == seen {
                let (g, timeout) = hub.cond.wait_timeout(s, Duration::from_millis(250)).unwrap();
                s = g;
                if timeout.timed_out() {
                    break;
                }
            }
            seen = s.version;
            if s.epoch != epoch {
                epoch = s.epoch;
                seen_tiles = 0;
                seen_walls = 0;
            }
            let walls = (s.walls_version > seen_walls).then(|| s.walls.clone()).flatten();
            seen_walls = s.walls_version;
            let motion = (s.motion_version > seen_motion).then(|| s.motion.clone()).flatten();
            seen_motion = s.motion_version;
            let scan = (s.scan_version > seen_scan).then(|| s.scan.clone()).flatten();
            seen_scan = s.scan_version;
            let zones = (s.zones_version > seen_zones).then(|| s.zones.clone()).flatten();
            seen_zones = s.zones_version;
            let stats = (s.stats_version > seen_stats).then(|| s.stats.clone()).flatten();
            seen_stats = s.stats_version;
            let mut tiles = None;
            if let Some((res, ox, oy, tile, tps)) = s.grid_header {
                let changed: Vec<Arc<Vec<u8>>> = s
                    .tile_version
                    .iter()
                    .zip(&s.tiles)
                    .filter(|(v, _)| **v > seen_tiles)
                    .filter_map(|(_, t)| t.clone())
                    .collect();
                if !changed.is_empty() || seen_tiles == 0 {
                    let mut b = Vec::with_capacity(20 + changed.iter().map(|t| t.len()).sum::<usize>());
                    b.extend_from_slice(&[3, 0]);
                    b.extend_from_slice(&(changed.len() as u16).to_le_bytes());
                    for v in [res, ox, oy] {
                        b.extend_from_slice(&v.to_le_bytes());
                    }
                    b.extend_from_slice(&tile.to_le_bytes());
                    b.extend_from_slice(&tps.to_le_bytes());
                    for t in &changed {
                        b.extend_from_slice(t);
                    }
                    tiles = Some(b);
                }
                seen_tiles = s.version;
            }
            (scan, zones, stats, tiles, walls, motion)
        };
        if let Some(w) = walls {
            ws.send(Message::Binary(w.to_vec())).map_err(err)?;
        }
        if let Some(m) = motion {
            ws.send(Message::Binary(m.to_vec())).map_err(err)?;
        }
        if let Some(t) = tiles {
            ws.send(Message::Binary(t)).map_err(err)?;
        }
        if let Some(z) = zones {
            ws.send(Message::Binary(z.to_vec())).map_err(err)?;
        }
        if let Some(sc) = scan {
            ws.send(Message::Binary(sc.to_vec())).map_err(err)?;
        }
        if let Some(st) = stats {
            ws.send(Message::Text(st.to_string())).map_err(err)?;
        }
        // Commands from the page (non-blocking).
        match ws.read() {
            Ok(Message::Text(t)) if t.trim() == "reset" => hub.reset_requested.store(true, Ordering::Relaxed),
            Ok(Message::Text(t)) if t.starts_with("rec:") => hub.commands.lock().unwrap().push(t.to_string()),
            Ok(Message::Close(_)) => return Ok(()),
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
            Err(e) => return Err(err(e)),
        }
    }
}
