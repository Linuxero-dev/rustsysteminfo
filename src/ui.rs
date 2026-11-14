use std::io::{self, IsTerminal};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Off,
    Ansi16,
    True,
}

fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| {
        let env = |k: &str| std::env::var(k).unwrap_or_default().to_ascii_lowercase();
        let (term, colorterm) = (env("TERM"), env("COLORTERM"));
        if !io::stdout().is_terminal() || term == "dumb" || !env("NO_COLOR").is_empty() {
            Mode::Off
        } else if colorterm.contains("truecolor") || colorterm.contains("24bit") {
            Mode::True
        } else if matches!(
            term.as_str(),
            "linux" | "vt100" | "vt220" | "xterm" | "screen"
        ) {
            Mode::Ansi16
        } else {
            Mode::True
        }
    })
}

fn sgr(code: &'static str) -> &'static str {
    if mode() == Mode::Off {
        ""
    } else {
        code
    }
}

pub fn reset() -> &'static str {
    sgr("\x1b[0m")
}
pub fn bold() -> &'static str {
    sgr("\x1b[1m")
}
pub fn dim() -> &'static str {
    sgr("\x1b[2m")
}

pub fn rgb(r: u8, g: u8, b: u8) -> String {
    match mode() {
        Mode::Off => String::new(),
        Mode::True => format!("\x1b[38;2;{r};{g};{b}m"),
        Mode::Ansi16 => {
            let bright = if r.max(g).max(b) > 150 { 60 } else { 0 };
            let base = 30 + (r > 110) as u8 + 2 * (g > 110) as u8 + 4 * (b > 110) as u8;
            format!("\x1b[{}m", base + bright)
        }
    }
}

pub fn pct_color(pct: f64) -> String {
    let (r, g, b) = if pct < 60.0 {
        (80, 250, 123)
    } else if pct < 85.0 {
        (241, 250, 140)
    } else {
        (255, 85, 85)
    };
    rgb(r, g, b)
}

fn unit(f: f64) -> f64 {
    if f.is_finite() {
        f.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn bar(fraction: f64, width: usize) -> String {
    let f = unit(fraction);
    let n = (f * width as f64).round() as usize;
    format!(
        "{}{}{}{}{}",
        pct_color(f * 100.0),
        "█".repeat(n),
        rgb(88, 91, 112),
        "░".repeat(width - n),
        reset()
    )
}

pub fn spark(values: &[f64], width: usize) -> String {
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut s = " ".repeat(width.saturating_sub(values.len()));
    for &v in &values[values.len().saturating_sub(width)..] {
        let f = unit(v);
        s += &format!(
            "{}{}",
            pct_color(f * 100.0),
            BLOCKS[(f * 7.0).round() as usize]
        );
    }
    s + reset()
}

pub fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(
                    c,
                    '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
                )
        })
        .collect()
}

fn scan(s: &str, max: usize) -> (String, usize) {
    let (mut out, mut n) = (String::with_capacity(s.len()), 0);
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            out.push(c);
            for d in it.by_ref() {
                out.push(d);
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
        } else if !c.is_control() {
            if n == max {
                break;
            }
            out.push(c);
            n += 1;
        }
    }
    (out, n)
}

pub fn visible_len(s: &str) -> usize {
    scan(s, usize::MAX).1
}

pub fn fit_width(s: &str, max: usize) -> String {
    let mut out = scan(s, max).0;
    out.push_str(reset());
    out
}

pub fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max.saturating_sub(1)).chain(['…']).collect()
    }
}

pub fn palette() -> Vec<String> {
    if mode() == Mode::Off {
        return Vec::new();
    }
    [40u8, 100]
        .map(|base| {
            (0..8)
                .map(|i| format!("\x1b[{}m  ", base + i))
                .collect::<String>()
                + reset()
        })
        .to_vec()
}

pub fn human(bytes: u64, prec: usize) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let (mut v, mut i) = (bytes as f64, 0);
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.prec$} {}", UNITS[i])
    }
}
