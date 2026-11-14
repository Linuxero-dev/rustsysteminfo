use crate::ui::{bar, bold, dim, fit_width, human, pct_color, reset, spark};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

const HISTORY: usize = 12;
const FRAME: Duration = Duration::from_millis(50);
const SAMPLE: Duration = Duration::from_millis(500);

pub type Row = (String, String);

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
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        if self.cpu.len() == HISTORY {
            self.cpu.remove(0);
        }
        self.cpu
            .push(self.sys.global_cpu_info().cpu_usage() as f64 / 100.0);
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

fn out(bytes: &[u8]) {
    let mut o = io::stdout().lock();
    let _ = o.write_all(bytes);
    let _ = o.flush();
}

struct RawGuard;

impl RawGuard {
    fn new() -> Option<Self> {
        terminal::enable_raw_mode().ok()?;
        out(b"\x1b[?1049h\x1b[?25l\x1b[?7l");
        Some(Self)
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        out(b"\x1b[?7h\x1b[?25h\x1b[?1049l");
        let _ = terminal::disable_raw_mode();
    }
}

fn size() -> (usize, usize) {
    let (c, r) = terminal::size().unwrap_or((80, 24));
    ((c as usize).max(20), (r as usize).max(3))
}

fn footer(paused: bool) -> String {
    let text = if paused {
        "PAUSADO: selecciona y copia con Ctrl+Shift+C · Espacio: reanudar · q: salir"
    } else {
        "Espacio: pausar para copiar texto · Enter, Esc o q: salir"
    };
    format!("{}{text}{}", dim(), reset())
}

fn draw(frame: &[Row], paused: bool, cols: usize, rows: usize, full: bool) -> usize {
    let w = cols.saturating_sub(1);
    let n = frame.len().min(rows.saturating_sub(2));
    let mut buf = String::from("\x1b[?2026h");
    if full {
        buf += "\x1b[H\x1b[2J";
    }
    for (i, (live, fixed)) in frame.iter().take(n).enumerate() {
        if !full && live.is_empty() {
            continue;
        }
        buf += &format!("\x1b[{};1H", i + 1);
        buf += &if full {
            fit_width(&format!("{live}{fixed}"), w)
        } else {
            fit_width(live, w)
        };
    }
    if full {
        buf += &format!("\x1b[{};1H{}", n + 2, fit_width(&footer(paused), w));
    }
    buf += "\x1b[?2026l";
    out(buf.as_bytes());
    n + 2
}

fn is_quit(k: &KeyEvent) -> bool {
    matches!(
        k.code,
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
    ) || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL))
}

fn print_once(mon: &mut Monitor, render: &mut impl FnMut(&Monitor, f64, usize, usize) -> Vec<Row>) {
    mon.sample();
    thread::sleep(Duration::from_millis(250));
    mon.sample();
    let cols = if io::stdout().is_terminal() {
        size().0
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
    mut render: impl FnMut(&Monitor, f64, usize, usize) -> Vec<Row>,
) {
    let guard = if interactive { RawGuard::new() } else { None };
    if guard.is_none() {
        return print_once(mon, &mut render);
    }
    let mut start = Instant::now();
    let mut last = start;
    let mut paused_at: Option<Instant> = None;
    let (mut seen, mut footer_row, mut full) = ((0, 0), 0, true);
    mon.sample();
    loop {
        let (cols, rows) = size();
        if (cols, rows) != seen {
            seen = (cols, rows);
            full = true;
        }
        let paused = paused_at.is_some();
        if !paused && last.elapsed() >= SAMPLE {
            mon.sample();
            last = Instant::now();
        }
        if !paused || full {
            let t = paused_at.unwrap_or_else(Instant::now).duration_since(start);
            let phase = if animate { t.as_secs_f64() * 7.5 } else { 0.0 };
            footer_row = draw(&render(mon, phase, cols, rows), paused, cols, rows, full);
            full = false;
        }
        let wait = if paused {
            SAMPLE
        } else if animate {
            FRAME
        } else {
            SAMPLE.saturating_sub(last.elapsed())
        };
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
                        out(format!("\x1b[{footer_row};1H\x1b[2K{f}").as_bytes());
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Err(_) => break,
        }
    }
}
