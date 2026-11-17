use crate::cache::Cache;
use crate::ui::{bar, bold, dim, fit_width, human, pct_color, reset, spark};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, System};

/// Velocidad de la ola del logo en radianes por segundo (más bajo = más lenta).
const WAVE_SPEED: f64 = 3.0;
const HISTORY: usize = 12;
const FRAME: Duration = Duration::from_millis(50);
const SAMPLE: Duration = Duration::from_millis(500);
const CPU_WINDOW: Duration = Duration::from_millis(100);
const CPU_FRESH: u64 = 2;
const MIN_WAIT: Duration = Duration::from_millis(1);

pub type Row = (String, String);

static GOT_SIGTERM: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigterm(_: libc::c_int) {
    GOT_SIGTERM.store(true, Ordering::Relaxed);
}

/// Valores instantáneos de CPU y memoria (para la salida JSON).
pub struct Snapshot {
    pub cpu: f64,
    pub used: u64,
    pub total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

fn parse_jiffies(stat: &str) -> Option<(u64, u64)> {
    let mut it = stat.lines().next()?.split_whitespace();
    if it.next()? != "cpu" {
        return None;
    }
    let v: Vec<u64> = it.filter_map(|t| t.parse().ok()).collect();
    if v.len() < 4 {
        return None;
    }
    let total: u64 = v.iter().sum();
    Some((total, v[3] + v.get(4).copied().unwrap_or(0)))
}

fn jiffies() -> Option<(u64, u64)> {
    parse_jiffies(&fs::read_to_string("/proc/stat").ok()?)
}

fn usage(prev: (u64, u64), cur: (u64, u64)) -> Option<f64> {
    let total = cur.0.checked_sub(prev.0)?;
    let idle = cur.1.checked_sub(prev.1)?;
    if total == 0 || idle > total {
        return None;
    }
    Some((total - idle) as f64 / total as f64)
}

fn stored_sample(cache: &Cache) -> Option<(u64, u64)> {
    let v = cache.hit("cpu", "")?;
    let (t, i) = v.split_once(':')?;
    Some((t.parse().ok()?, i.parse().ok()?))
}

fn remember(cache: &mut Cache, s: (u64, u64)) {
    cache.store("cpu", "", &format!("{}:{}", s.0, s.1));
}

fn cpu_usage(cache: &mut Cache) -> f64 {
    let Some(cur) = jiffies() else { return 0.0 };
    if cache.age("cpu").is_some_and(|a| a <= CPU_FRESH) {
        if let Some(prev) = stored_sample(cache) {
            if let Some(u) = usage(prev, cur) {
                remember(cache, cur);
                return u;
            }
        }
    }
    thread::sleep(CPU_WINDOW);
    let next = jiffies().unwrap_or(cur);
    remember(cache, next);
    usage(cur, next).unwrap_or(0.0)
}

pub struct Monitor {
    sys: System,
    cpu: Vec<f64>,
}

impl Monitor {
    pub fn new(sys: System) -> Self {
        Self {
            sys,
            cpu: Vec::with_capacity(HISTORY),
        }
    }

    pub fn sample(&mut self) {
        // Solo el uso: la frecuencia por núcleo no se muestra en vivo y cuesta lecturas extra.
        self.sys
            .refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());
        self.sys.refresh_memory();
        if self.cpu.len() == HISTORY {
            self.cpu.remove(0);
        }
        self.cpu
            .push(self.sys.global_cpu_info().cpu_usage() as f64 / 100.0);
    }

    pub fn seed(&mut self, cpu: f64) {
        self.sys.refresh_memory();
        self.cpu.clear();
        self.cpu.push(cpu.clamp(0.0, 1.0));
    }

    pub fn measure(&mut self, cache: &mut Cache) -> Snapshot {
        let cpu = cpu_usage(cache);
        self.seed(cpu);
        Snapshot {
            cpu,
            used: self.sys.used_memory(),
            total: self.sys.total_memory(),
            swap_used: self.sys.used_swap(),
            swap_total: self.sys.total_swap(),
        }
    }

    pub fn lines(&self, accent: &str) -> Vec<String> {
        let (b, d, r) = (bold(), dim(), reset());
        let meter = |name: &str, f: f64, tail: String| {
            format!(
                "{accent}{b}{name:<4}{r} {} {}{:>3.0}%{r} {tail}",
                bar(f, 12),
                pct_color(f * 100.0),
                f * 100.0
            )
        };
        let frac = |used: u64, total: u64| {
            if total > 0 {
                used as f64 / total as f64
            } else {
                0.0
            }
        };
        let mem = |used: u64, total: u64| format!("{}/{}", human(used, 1), human(total, 1));
        let s = &self.sys;
        let (used, total, sused, stotal) = (
            s.used_memory(),
            s.total_memory(),
            s.used_swap(),
            s.total_swap(),
        );
        let swap = if stotal > 0 {
            meter("Swap", frac(sused, stotal), mem(sused, stotal))
        } else {
            format!("{accent}{b}{:<4}{r} {d}desactivado{r}", "Swap")
        };
        vec![
            meter(
                "CPU",
                self.cpu.last().copied().unwrap_or(0.0),
                spark(&self.cpu, 8),
            ),
            meter("RAM", frac(used, total), mem(used, total)),
            swap,
        ]
    }
}

fn out(bytes: &[u8]) -> bool {
    let mut o = io::stdout().lock();
    o.write_all(bytes).is_ok() && o.flush().is_ok()
}

struct RawGuard;

impl RawGuard {
    fn new() -> Option<Self> {
        terminal::enable_raw_mode().ok()?;
        if !out(b"\x1b[?1049h\x1b[?25l\x1b[?7l") {
            let _ = terminal::disable_raw_mode();
            return None;
        }
        Some(Self)
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = out(b"\x1b[?7h\x1b[?25h\x1b[?1049l");
        let _ = terminal::disable_raw_mode();
    }
}

fn size() -> Option<(usize, usize)> {
    let (c, r) = terminal::size().ok()?;
    Some(((c as usize).max(20), (r as usize).max(3)))
}

fn wait_for(paused: bool, animate: bool, since_draw: Duration, since_sample: Duration) -> Duration {
    let base = if paused {
        SAMPLE
    } else if animate {
        FRAME.saturating_sub(since_draw)
    } else {
        SAMPLE.saturating_sub(since_sample)
    };
    base.max(MIN_WAIT)
}

fn footer(paused: bool) -> String {
    let text = if paused {
        "PAUSADO: selecciona y copia con Ctrl+Shift+C · Espacio: reanudar · q: salir"
    } else {
        "Espacio: pausar para copiar texto · Enter, Esc o q: salir"
    };
    format!("{}{text}{}", dim(), reset())
}

fn dirty(prev: &[Row], frame: &[Row], n: usize, full: bool) -> Vec<usize> {
    (0..n)
        .filter(|&i| full || prev.get(i) != frame.get(i))
        .collect()
}

fn draw(
    frame: &[Row],
    prev: &mut Vec<Row>,
    paused: bool,
    cols: usize,
    rows: usize,
    full: bool,
) -> Option<usize> {
    let w = cols.saturating_sub(1);
    let n = frame.len().min(rows.saturating_sub(2));
    let mut buf = String::with_capacity(n * (w + 16) + 32);
    buf += "\x1b[?2026h";
    if full {
        buf += "\x1b[H\x1b[2J";
    }
    for i in dirty(prev, frame, n, full) {
        let (live, fixed) = &frame[i];
        buf += &format!("\x1b[{};1H\x1b[2K", i + 1);
        buf += &fit_width(&format!("{live}{fixed}"), w);
    }
    buf += &format!("\x1b[{};1H\x1b[2K{}", n + 2, fit_width(&footer(paused), w));
    buf += "\x1b[?2026l";
    if !out(buf.as_bytes()) {
        return None;
    }
    prev.clear();
    prev.extend(frame.iter().take(n).cloned());
    Some(n + 2)
}

fn is_quit(k: &KeyEvent) -> bool {
    matches!(
        k.code,
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
    ) || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL))
}

fn print_once(
    mon: &mut Monitor,
    cache: &mut Cache,
    render: &mut impl FnMut(&Monitor, f64, usize, usize) -> Vec<Row>,
) {
    mon.seed(cpu_usage(cache));
    let cols = if io::stdout().is_terminal() {
        size().map(|(c, _)| c).unwrap_or(80)
    } else {
        usize::MAX
    };
    let mut o = io::stdout().lock();
    for (live, fixed) in render(mon, 0.0, cols, usize::MAX) {
        if writeln!(
            o,
            "{}",
            fit_width(&format!("{live}{fixed}"), cols.saturating_sub(1))
        )
        .is_err()
        {
            break;
        }
    }
}

pub fn run(
    mon: &mut Monitor,
    animate: bool,
    interactive: bool,
    cache: &mut Cache,
    mut render: impl FnMut(&Monitor, f64, usize, usize) -> Vec<Row>,
) {
    let guard = if interactive { RawGuard::new() } else { None };
    if guard.is_none() {
        return print_once(mon, cache, &mut render);
    }
    // Los manejadores se instalan solo si hay modo interactivo real: si falla el modo raw,
    // se imprime una vez y Ctrl+C conserva su comportamiento por defecto.
    // SAFETY: el manejador únicamente escribe un AtomicBool, seguro dentro de una señal.
    unsafe {
        for sig in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
            libc::signal(sig, on_sigterm as *const () as libc::sighandler_t);
        }
    }

    let mut start = Instant::now();
    let mut last = start;
    let mut last_draw = start;
    let mut paused_at: Option<Instant> = None;
    let (mut seen, mut footer_row, mut full) = ((0, 0), 0, true);
    let mut prev_len = 0;
    let mut prev: Vec<Row> = Vec::new();
    mon.seed(cpu_usage(cache));

    loop {
        if GOT_SIGTERM.load(Ordering::Relaxed) {
            break;
        }
        let Some((cols, rows)) = size() else { break };
        if (cols, rows) != seen {
            seen = (cols, rows);
            full = true;
        }
        let paused = paused_at.is_some();
        let mut sampled = false;
        if !paused && last.elapsed() >= SAMPLE {
            mon.sample();
            last = Instant::now();
            sampled = true;
        }
        // Solo se redibuja cuando toca (muestra nueva o fotograma de la animación):
        // los eventos del ratón o del teclado no deben provocar un dibujo cada uno.
        let tick = animate && last_draw.elapsed() >= FRAME;
        if full || (!paused && (sampled || tick)) {
            let t = paused_at.unwrap_or_else(Instant::now).duration_since(start);
            let phase = if animate {
                t.as_secs_f64() * WAVE_SPEED
            } else {
                0.0
            };
            let frame = render(mon, phase, cols, rows);
            if frame.len() != prev_len {
                prev_len = frame.len();
                full = true;
            }
            let Some(row) = draw(&frame, &mut prev, paused, cols, rows, full) else {
                break;
            };
            footer_row = row;
            full = false;
            last_draw = Instant::now();
        }
        let wait = wait_for(paused, animate, last_draw.elapsed(), last.elapsed());
        match event::poll(wait) {
            Ok(false) => {}
            Ok(true) => match event::read() {
                Ok(Event::Key(k)) if k.kind != KeyEventKind::Release => {
                    if is_quit(&k) {
                        break;
                    }
                    if k.code == KeyCode::Char(' ') {
                        match paused_at.take() {
                            Some(t) => start += t.elapsed(),
                            None => paused_at = Some(Instant::now()),
                        }
                        let f = fit_width(&footer(paused_at.is_some()), cols.saturating_sub(1));
                        if !out(format!("\x1b[{footer_row};1H\x1b[2K{f}").as_bytes()) {
                            break;
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn row(s: &str) -> Row {
        (s.to_string(), String::new())
    }

    #[test]
    fn quit_keys_are_enter_esc_q_and_ctrl_c() {
        for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Char('q')] {
            assert!(is_quit(&key(code, KeyModifiers::NONE)));
        }
        assert!(is_quit(&key(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!is_quit(&key(KeyCode::Char('c'), KeyModifiers::NONE)));
        assert!(!is_quit(&key(KeyCode::Char(' '), KeyModifiers::NONE)));
    }

    #[test]
    fn footer_changes_with_the_paused_state() {
        assert!(footer(true).contains("PAUSADO"));
        assert!(!footer(false).contains("PAUSADO"));
    }

    #[test]
    fn jiffies_sum_every_field_and_count_iowait_as_idle() {
        let s = "cpu  10 0 10 70 10 0 0 0 0 0\ncpu0 1 0 1 7 0 0 0 0 0 0\nintr 1\n";
        assert_eq!(parse_jiffies(s), Some((100, 80)));
        assert_eq!(parse_jiffies("cpu  1 2\n"), None);
        assert_eq!(parse_jiffies("procs_running 2\n"), None);
        assert_eq!(parse_jiffies(""), None);
    }

    #[test]
    fn usage_rejects_resets_and_empty_windows() {
        assert_eq!(usage((100, 80), (200, 130)), Some(0.5));
        assert_eq!(usage((200, 130), (100, 80)), None);
        assert_eq!(usage((100, 80), (100, 80)), None);
        assert_eq!(usage((100, 80), (200, 250)), None);
    }

    #[test]
    fn seed_clamps_the_value_and_restarts_the_history() {
        let mut m = Monitor::new(System::new());
        m.seed(2.0);
        assert_eq!(m.cpu.last().copied(), Some(1.0));
        m.seed(-1.0);
        assert_eq!(m.cpu.len(), 1);
        assert_eq!(m.cpu.last().copied(), Some(0.0));
    }

    #[test]
    fn only_changed_rows_are_repainted() {
        let a = vec![row("uno"), row("dos"), row("tres")];
        let b = vec![row("uno"), row("cambio"), row("tres")];
        assert!(dirty(&a, &a, 3, false).is_empty());
        assert_eq!(dirty(&a, &b, 3, false), vec![1]);
        assert_eq!(dirty(&a, &b, 3, true), vec![0, 1, 2]);
        assert_eq!(dirty(&[], &b, 3, false), vec![0, 1, 2]);
        assert_eq!(dirty(&a, &b, 1, false), Vec::<usize>::new());
    }

    #[test]
    fn the_loop_never_waits_less_than_the_floor() {
        assert_eq!(wait_for(false, true, Duration::ZERO, Duration::ZERO), FRAME);
        assert_eq!(wait_for(false, true, FRAME * 2, Duration::ZERO), MIN_WAIT);
        assert_eq!(wait_for(true, true, FRAME * 2, Duration::ZERO), SAMPLE);
        assert_eq!(wait_for(false, false, Duration::ZERO, SAMPLE * 2), MIN_WAIT);
    }
}
