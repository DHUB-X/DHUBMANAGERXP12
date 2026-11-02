//! Stutter and Frame Drop Detector for X-Plane
//! Listens to live UDP performance packets (Group 0: Frame Rate) from X-Plane,
//! tracks severe frame drops (< 15 FPS), diagnoses the bottleneck (CPU vs GPU vs VRAM vs Scenery I/O),
//! and records real-time history for simulator performance analysis.

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_PORT: u16 = 49005;
const DEFAULT_THRESHOLD_FPS: f32 = 15.0;
const MAX_HISTORY_EVENTS: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StutterEvent {
    pub id: String,
    pub timestamp: i64,
    pub time_formatted: String,
    pub lowest_fps: f32,
    pub baseline_fps: f32,
    pub duration_seconds: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub vram_usage_percent: Option<f32>,
    pub primary_cause: String,
    pub cause_title: String,
    pub cause_description: String,
    pub suggestion: String,
    pub context_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StutterMonitorStatus {
    pub is_listening: bool,
    pub is_connected: bool,
    pub port: u16,
    pub current_fps: f32,
    pub avg_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub one_percent_low_fps: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub threshold_fps: f32,
    pub total_drops_recorded: usize,
    pub lowest_drop_recorded: Option<f32>,
    pub last_packet_time: Option<i64>,
}

struct InProgressDrop {
    start_time: Instant,
    last_commit_time: Instant,
    baseline_fps: f32,
    lowest_fps: f32,
    peak_cpu_ms: f32,
    peak_gpu_ms: f32,
    logged_event_id: Option<String>,
}

struct DetectorInner {
    events: Vec<StutterEvent>,
    current_fps: f32,
    cpu_time_ms: f32,
    gpu_time_ms: f32,
    threshold_fps: f32,
    last_packet_instant: Option<Instant>,
    last_packet_epoch: Option<i64>,
    active_drop: Option<InProgressDrop>,
    xplane_path: Option<PathBuf>,
    total_frames_sampled: u64,
    sum_fps: f64,
    min_fps: f32,
    max_fps: f32,
    recent_fps_window: Vec<f32>,
}

pub struct StutterDetector {
    inner: Arc<Mutex<DetectorInner>>,
    is_running: Arc<AtomicBool>,
    port: Arc<AtomicU16>,
}

static DETECTOR_INSTANCE: Mutex<Option<Arc<StutterDetector>>> = Mutex::new(None);

pub fn get_or_init_detector() -> Arc<StutterDetector> {
    let mut guard = DETECTOR_INSTANCE.lock().unwrap();
    if let Some(ref detector) = *guard {
        return detector.clone();
    }

    let detector = Arc::new(StutterDetector {
        inner: Arc::new(Mutex::new(DetectorInner {
            events: Vec::new(),
            current_fps: 0.0,
            cpu_time_ms: 0.0,
            gpu_time_ms: 0.0,
            threshold_fps: DEFAULT_THRESHOLD_FPS,
            last_packet_instant: None,
            last_packet_epoch: None,
            active_drop: None,
            xplane_path: None,
            total_frames_sampled: 0,
            sum_fps: 0.0,
            min_fps: 0.0,
            max_fps: 0.0,
            recent_fps_window: Vec::with_capacity(1000),
        })),
        is_running: Arc::new(AtomicBool::new(false)),
        port: Arc::new(AtomicU16::new(DEFAULT_PORT)),
    });

    *guard = Some(detector.clone());
    detector
}

impl StutterDetector {
    pub fn start(&self, port: Option<u16>, threshold: Option<f32>, xplane_path: Option<String>) -> Result<bool, String> {
        let target_port = port.unwrap_or(DEFAULT_PORT);
        self.port.store(target_port, Ordering::SeqCst);

        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(th) = threshold {
                inner.threshold_fps = th;
            }
            if let Some(xp) = xplane_path {
                inner.xplane_path = Some(PathBuf::from(xp));
            }
        }

        if self.is_running.load(Ordering::SeqCst) {
            return Ok(true);
        }

        self.is_running.store(true, Ordering::SeqCst);
        let is_running_clone = self.is_running.clone();
        let inner_clone = self.inner.clone();

        thread::spawn(move || {
            let bind_addr = format!("0.0.0.0:{}", target_port);
            let socket = match UdpSocket::bind(&bind_addr) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[StutterDetector] Failed to bind UDP on {}: {}", bind_addr, e);
                    is_running_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };

            // Set read timeout so thread can gracefully check is_running
            let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));
            let mut buf = [0u8; 1500];

            while is_running_clone.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buf) {
                    Ok((amt, _src)) => {
                        if amt >= 5 && &buf[0..4] == b"DATA" {
                            Self::parse_data_packet(&buf[5..amt], &inner_clone);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        // Check if active drop has timed out without recovery
                        Self::check_drop_timeout(&inner_clone);
                    }
                    Err(e) => {
                        eprintln!("[StutterDetector] UDP receive error: {}", e);
                    }
                }
            }
        });

        Ok(true)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn set_threshold(&self, threshold: f32) {
        let mut inner = self.inner.lock().unwrap();
        inner.threshold_fps = threshold;
    }

    #[allow(dead_code)]
    pub fn set_xplane_path(&self, path: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.xplane_path = Some(PathBuf::from(path));
    }

    pub fn get_status(&self) -> StutterMonitorStatus {
        let inner = self.inner.lock().unwrap();
        let now = Instant::now();
        let is_connected = match inner.last_packet_instant {
            Some(last) => now.duration_since(last) < Duration::from_secs(3),
            None => false,
        };

        let lowest_drop = inner
            .events
            .iter()
            .map(|e| e.lowest_fps)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg_fps = if inner.total_frames_sampled > 0 {
            ((inner.sum_fps / inner.total_frames_sampled as f64 * 10.0).round() / 10.0) as f32
        } else {
            0.0
        };

        let min_fps = if inner.min_fps > 0.0 {
            (inner.min_fps * 10.0).round() / 10.0
        } else {
            0.0
        };

        let max_fps = (inner.max_fps * 10.0).round() / 10.0;

        let one_percent_low_fps = if !inner.recent_fps_window.is_empty() {
            let mut sorted = inner.recent_fps_window.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let idx = ((sorted.len() as f32 * 0.01).floor() as usize).min(sorted.len() - 1);
            (sorted[idx] * 10.0).round() / 10.0
        } else {
            0.0
        };

        StutterMonitorStatus {
            is_listening: self.is_running.load(Ordering::SeqCst),
            is_connected,
            port: self.port.load(Ordering::SeqCst),
            current_fps: inner.current_fps,
            avg_fps,
            min_fps,
            max_fps,
            one_percent_low_fps,
            cpu_time_ms: inner.cpu_time_ms,
            gpu_time_ms: inner.gpu_time_ms,
            threshold_fps: inner.threshold_fps,
            total_drops_recorded: inner.events.len(),
            lowest_drop_recorded: lowest_drop,
            last_packet_time: inner.last_packet_epoch,
        }
    }

    pub fn get_events(&self) -> Vec<StutterEvent> {
        let inner = self.inner.lock().unwrap();
        inner.events.clone()
    }

    pub fn clear_events(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.events.clear();
        inner.total_frames_sampled = 0;
        inner.sum_fps = 0.0;
        inner.min_fps = 0.0;
        inner.max_fps = 0.0;
        inner.recent_fps_window.clear();
    }

    pub fn simulate_event(&self, scenario: Option<String>) -> StutterEvent {
        let mut inner = self.inner.lock().unwrap();
        let scenario = scenario.unwrap_or_else(|| "vram".to_string());

        let (lowest, cpu, gpu, vram, cause, title, desc, sugg) = match scenario.as_str() {
            "cpu" => (
                10.8,
                64.5,
                14.2,
                Some(68.0),
                "cpu_bottleneck",
                "CPU Bottleneck (Main Thread / Scripts)",
                "Processor frame time spiked (64.5ms) while GPU was idle. Main thread is overloaded by aircraft plugins, AI traffic, or world object calculations.",
                "Reduce World Objects density, decrease AI traffic (CSL/LiveTraffic), or check heavy FlyWithLua scripts."
            ),
            "gpu" => (
                11.2,
                15.4,
                72.0,
                Some(75.0),
                "gpu_overload",
                "GPU Overload (Heavy Visual Rendering)",
                "Graphics card was overloaded rendering frames (72.0ms), typically triggered by thick volumetric clouds, storm weather, or high MSAA.",
                "Lower Anti-Aliasing (MSAA) or decrease cloud rendering quality in Graphics Settings."
            ),
            "scenery" => (
                9.5,
                38.0,
                32.0,
                Some(72.0),
                "scenery_disk_io",
                "Scenery Disk I/O & Tile Loading",
                "A brief pause occurred while X-Plane streamed terrain DSF mesh or new airport packs from disk into system memory.",
                "Short 1-2 second pauses entering dense areas are common on HDDs. Moving scenery to an SSD eliminates this pause."
            ),
            _ => (
                8.4,
                24.5,
                22.0,
                Some(96.5),
                "vram_overload",
                "VRAM Overload (GPU Texture Spillover)",
                "GPU VRAM usage reached 96.5%. Windows was forced to page textures into system RAM over PCIe, causing a severe framerate collapse.",
                "Lower Texture Quality from Maximum to High in X-Plane Graphics Settings to save 2-4 GB VRAM."
            )
        };

        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let event = StutterEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now_epoch,
            time_formatted: Local::now().format("%H:%M:%S").to_string(),
            lowest_fps: lowest,
            baseline_fps: 44.5,
            duration_seconds: 2.8,
            cpu_time_ms: cpu,
            gpu_time_ms: gpu,
            vram_usage_percent: vram,
            primary_cause: cause.to_string(),
            cause_title: title.to_string(),
            cause_description: desc.to_string(),
            suggestion: sugg.to_string(),
            context_notes: Some("Diagnostic test simulation.".to_string()),
        };

        inner.events.insert(0, event.clone());
        if inner.events.len() > MAX_HISTORY_EVENTS {
            inner.events.pop();
        }

        event
    }

    fn parse_data_packet(data: &[u8], inner_lock: &Arc<Mutex<DetectorInner>>) {
        let chunk_size = 36;
        let num_records = data.len() / chunk_size;

        let now_instant = Instant::now();
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for i in 0..num_records {
            let offset = i * chunk_size;
            let record = &data[offset..offset + chunk_size];

            let index = i32::from_le_bytes([record[0], record[1], record[2], record[3]]);
            if index == 0 {
                // Group 0: Frame Rate
                let f_act = f32::from_le_bytes([record[4], record[5], record[6], record[7]]);
                let cpu_sec = f32::from_le_bytes([record[16], record[17], record[18], record[19]]);
                let gpu_sec = f32::from_le_bytes([record[20], record[21], record[22], record[23]]);

                let cpu_ms = (cpu_sec * 1000.0).max(0.0);
                let gpu_ms = (gpu_sec * 1000.0).max(0.0);

                let mut inner = inner_lock.lock().unwrap();
                inner.current_fps = f_act;
                inner.cpu_time_ms = cpu_ms;
                inner.gpu_time_ms = gpu_ms;
                inner.last_packet_instant = Some(now_instant);
                inner.last_packet_epoch = Some(now_epoch);

                if f_act > 0.0 {
                    inner.total_frames_sampled += 1;
                    inner.sum_fps += f_act as f64;
                    if inner.min_fps == 0.0 || f_act < inner.min_fps {
                        inner.min_fps = f_act;
                    }
                    if f_act > inner.max_fps {
                        inner.max_fps = f_act;
                    }
                    inner.recent_fps_window.push(f_act);
                    if inner.recent_fps_window.len() > 1000 {
                        inner.recent_fps_window.remove(0);
                    }
                }

                let threshold = inner.threshold_fps;

                                // Check frame drop logic
                if f_act < threshold && f_act > 0.5 {
                    let mut should_create = false;
                    let mut should_update: Option<(String, f32, f32, f32, f32)> = None;
                    let mut drop_snapshot: Option<(InProgressDrop, f32)> = None;

                    if let Some(ref mut drop) = inner.active_drop {
                        drop.lowest_fps = drop.lowest_fps.min(f_act);
                        drop.peak_cpu_ms = drop.peak_cpu_ms.max(cpu_ms);
                        drop.peak_gpu_ms = drop.peak_gpu_ms.max(gpu_ms);

                        let elapsed = drop.start_time.elapsed().as_secs_f32();
                        let since_commit = drop.last_commit_time.elapsed().as_secs_f32();

                        if elapsed >= 1.0 && drop.logged_event_id.is_none() {
                            should_create = true;
                            drop.last_commit_time = now_instant;
                            drop_snapshot = Some((
                                InProgressDrop {
                                    start_time: drop.start_time,
                                    last_commit_time: drop.last_commit_time,
                                    baseline_fps: drop.baseline_fps,
                                    lowest_fps: drop.lowest_fps,
                                    peak_cpu_ms: drop.peak_cpu_ms,
                                    peak_gpu_ms: drop.peak_gpu_ms,
                                    logged_event_id: None,
                                },
                                elapsed,
                            ));
                        } else if since_commit >= 3.5 {
                            if let Some(ref target_id) = drop.logged_event_id {
                                should_update = Some((
                                    target_id.clone(),
                                    (elapsed * 10.0).round() / 10.0,
                                    (drop.lowest_fps * 10.0).round() / 10.0,
                                    (drop.peak_cpu_ms * 10.0).round() / 10.0,
                                    (drop.peak_gpu_ms * 10.0).round() / 10.0,
                                ));
                            }
                            drop.last_commit_time = now_instant;
                        }
                    } else {
                        let baseline = if inner.current_fps >= threshold { inner.current_fps } else { 35.0 };
                        inner.active_drop = Some(InProgressDrop {
                            start_time: now_instant,
                            last_commit_time: now_instant,
                            baseline_fps: baseline,
                            lowest_fps: f_act,
                            peak_cpu_ms: cpu_ms,
                            peak_gpu_ms: gpu_ms,
                            logged_event_id: None,
                        });
                    }

                    if should_create {
                        if let Some((drop_snap, elapsed)) = drop_snapshot {
                            let xp_path = inner.xplane_path.clone();
                            let event = Self::create_drop_event(&drop_snap, elapsed, now_epoch, xp_path.as_deref());
                            let ev_id = event.id.clone();
                            if let Some(ref mut drop) = inner.active_drop {
                                drop.logged_event_id = Some(ev_id);
                            }
                            inner.events.insert(0, event);
                            if inner.events.len() > MAX_HISTORY_EVENTS {
                                inner.events.pop();
                            }
                        }
                    } else if let Some((target_id, dur, low, cpu_v, gpu_v)) = should_update {
                        if let Some(ev) = inner.events.iter_mut().find(|e| e.id == target_id) {
                            ev.duration_seconds = dur;
                            ev.lowest_fps = low;
                            ev.cpu_time_ms = cpu_v;
                            ev.gpu_time_ms = gpu_v;
                        }
                    }
                } else if f_act >= threshold {
                    if let Some(drop) = inner.active_drop.take() {
                        let duration = drop.start_time.elapsed().as_secs_f32();
                        if let Some(ref target_id) = drop.logged_event_id {
                            // Finalize duration of already-committed drop
                            if let Some(ev) = inner.events.iter_mut().find(|e| e.id == *target_id) {
                                ev.duration_seconds = (duration * 10.0).round() / 10.0;
                                ev.lowest_fps = (drop.lowest_fps * 10.0).round() / 10.0;
                            }
                        } else if duration >= 0.4 {
                            let xp_path = inner.xplane_path.clone();
                            let event = Self::create_drop_event(&drop, duration, now_epoch, xp_path.as_deref());
                            inner.events.insert(0, event);
                            if inner.events.len() > MAX_HISTORY_EVENTS {
                                inner.events.pop();
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_drop_timeout(inner_lock: &Arc<Mutex<DetectorInner>>) {
        let mut inner = inner_lock.lock().unwrap();
        if let Some(drop) = inner.active_drop.take() {
            let duration = drop.start_time.elapsed().as_secs_f32();
            if let Some(ref target_id) = drop.logged_event_id {
                if let Some(ev) = inner.events.iter_mut().find(|e| e.id == *target_id) {
                    ev.duration_seconds = (duration * 10.0).round() / 10.0;
                    ev.lowest_fps = (drop.lowest_fps * 10.0).round() / 10.0;
                }
            } else if duration >= 0.5 {
                let now_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let xp_path = inner.xplane_path.clone();
                let event = Self::create_drop_event(&drop, duration, now_epoch, xp_path.as_deref());
                inner.events.insert(0, event);
                if inner.events.len() > MAX_HISTORY_EVENTS {
                    inner.events.pop();
                }
            }
        }
    }

    fn create_drop_event(
        drop: &InProgressDrop,
        duration: f32,
        now_epoch: i64,
        xplane_path: Option<&Path>,
    ) -> StutterEvent {
        let (vram_percent, log_notes) = Self::sample_environment_diagnostics(xplane_path);

        let (cause, title, desc, sugg) = if vram_percent.unwrap_or(0.0) >= 92.0 {
            (
                "vram_overload",
                "VRAM Overload (GPU Texture Spillover)",
                format!(
                    "VRAM usage reached {:.1}%. Exhausted video memory forced texture paging into system RAM over PCIe.",
                    vram_percent.unwrap_or(92.0)
                ),
                "Lower Texture Quality from Maximum to High in X-Plane Graphics Settings to save 2-4 GB VRAM.".to_string(),
            )
        } else if let Some(ref notes) = log_notes {
            if notes.contains(".dsf") || notes.contains("scenery") || notes.contains("DSF") {
                (
                    "scenery_disk_io",
                    "Scenery Disk I/O & Tile Loading",
                    "Simulator read new scenery/tile packages from disk during flight transition.".to_string(),
                    "Brief pauses entering dense areas are common on mechanical HDDs. Moving custom scenery to SSD is recommended.".to_string(),
                )
            } else {
                Self::diagnose_cpu_gpu(drop.peak_cpu_ms, drop.peak_gpu_ms)
            }
        } else {
            Self::diagnose_cpu_gpu(drop.peak_cpu_ms, drop.peak_gpu_ms)
        };

        StutterEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now_epoch,
            time_formatted: Local::now().format("%H:%M:%S").to_string(),
            lowest_fps: (drop.lowest_fps * 10.0).round() / 10.0,
            baseline_fps: (drop.baseline_fps * 10.0).round() / 10.0,
            duration_seconds: (duration * 10.0).round() / 10.0,
            cpu_time_ms: (drop.peak_cpu_ms * 10.0).round() / 10.0,
            gpu_time_ms: (drop.peak_gpu_ms * 10.0).round() / 10.0,
            vram_usage_percent: vram_percent,
            primary_cause: cause.to_string(),
            cause_title: title.to_string(),
            cause_description: desc,
            suggestion: sugg,
            context_notes: log_notes,
        }
    }

    fn diagnose_cpu_gpu(cpu_ms: f32, gpu_ms: f32) -> (&'static str, &'static str, String, String) {
        if cpu_ms > gpu_ms * 1.35 || cpu_ms > 45.0 {
            (
                "cpu_bottleneck",
                "CPU Bottleneck (Main Thread / Scripts)",
                format!(
                    "Processor time (CPU: {:.1}ms) significantly exceeded graphics card time (GPU: {:.1}ms). Main thread was constrained by scenery or script execution.",
                    cpu_ms, gpu_ms
                ),
                "Reduce World Objects density, decrease AI traffic, or inspect heavy FlyWithLua scripts.".to_string(),
            )
        } else if gpu_ms > cpu_ms * 1.35 || gpu_ms > 45.0 {
            (
                "gpu_overload",
                "GPU Overload (Heavy Visual Rendering)",
                format!(
                    "Graphics card (GPU: {:.1}ms) was overloaded rendering visual geometry compared to processor (CPU: {:.1}ms).",
                    gpu_ms, cpu_ms
                ),
                "Lower Anti-Aliasing (MSAA), decrease volumetric cloud quality, or lower shadow resolution in Graphics Settings.".to_string(),
            )
        } else {
            (
                "general_heavy_scene",
                "Simultaneous CPU & GPU Saturation",
                format!(
                    "Both components experienced heavy load concurrently (CPU: {:.1}ms, GPU: {:.1}ms) in a dense scenery or adverse weather zone.",
                    cpu_ms, gpu_ms
                ),
                "Combination of complex airport assets and demanding weather. Lower object density slightly to restore headroom.".to_string(),
            )
        }
    }

    fn sample_environment_diagnostics(xplane_path: Option<&Path>) -> (Option<f32>, Option<String>) {
        let mut vram_percent = None;

        #[cfg(windows)]
        {
            #[repr(C)]
            struct MemoryStatusEx {
                dw_length: u32,
                dw_memory_load: u32,
                ull_total_phys: u64,
                ull_avail_phys: u64,
                ull_total_page_file: u64,
                ull_avail_page_file: u64,
                ull_total_virtual: u64,
                ull_avail_virtual: u64,
                ull_avail_extended_virtual: u64,
            }

            extern "system" {
                fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
            }

            let mut mem = MemoryStatusEx {
                dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
                dw_memory_load: 0,
                ull_total_phys: 0,
                ull_avail_phys: 0,
                ull_total_page_file: 0,
                ull_avail_page_file: 0,
                ull_total_virtual: 0,
                ull_avail_virtual: 0,
                ull_avail_extended_virtual: 0,
            };

            if unsafe { GlobalMemoryStatusEx(&mut mem) } != 0 {
                let load = mem.dw_memory_load as f32;
                if load > 80.0 {
                    vram_percent = Some(load);
                }
            }
        }

        let mut notes = None;
        if let Some(xp) = xplane_path {
            let log_file = xp.join("Log.txt");
            if log_file.exists() {
                if let Ok(metadata) = fs::metadata(&log_file) {
                    let file_size = metadata.len();
                    if let Ok(file_bytes) = fs::read(&log_file) {
                        let start = if file_size > 8192 { (file_size - 8192) as usize } else { 0 };
                        let snippet = String::from_utf8_lossy(&file_bytes[start..]);
                        let lines: Vec<&str> = snippet.lines().collect();
                        let recent_lines: Vec<&str> = lines.iter().rev().take(6).copied().collect();

                        for line in recent_lines {
                            if line.contains("DSF") || line.contains("Loading new scenery") || line.contains(".dsf") {
                                notes = Some(format!("Log: {}", line.trim()));
                                break;
                            }
                        }
                    }
                }
            }
        }

        (vram_percent, notes)
    }
}
