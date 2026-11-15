use std::io::{self, IsTerminal};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Off,
    Ansi16,
    Ansi256,
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
        } else if term.contains("256color") {
            Mode::Ansi256
        } else if term.is_empty() {
            Mode::Off
        } else {
            Mode::Ansi16
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

/// Color más cercano del cubo 6x6x6 de la paleta de 256 colores (índices 16 a 231).
fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    let level = |v: u8| ((v as u16 * 5 + 127) / 255) as u8;
    16 + 36 * level(r) + 6 * level(g) + level(b)
}

pub fn rgb(r: u8, g: u8, b: u8) -> String {
    match mode() {
        Mode::Off => String::new(),
        Mode::True => format!("\x1b[38;2;{r};{g};{b}m"),
        Mode::Ansi256 => format!("\x1b[38;5;{}m", rgb_to_256(r, g, b)),
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

fn skip_ansi_sequence<I: Iterator<Item = char>>(it: &mut std::iter::Peekable<I>) {
    match it.peek().copied() {
        Some('[') => {
            it.next();
            for ch in it.by_ref() {
                if ch.is_ascii() && (b'@'..=b'~').contains(&(ch as u8)) {
                    break;
                }
            }
        }
        Some(']') => {
            it.next();
            while let Some(ch) = it.next() {
                if ch == '\u{7}' || (ch == '\u{1b}' && it.next_if_eq(&'\\').is_some()) {
                    break;
                }
            }
        }
        Some('P') | Some('X') | Some('^') | Some('_') => {
            it.next();
            while let Some(ch) = it.next() {
                if ch == '\u{1b}' && it.next_if_eq(&'\\').is_some() {
                    break;
                }
            }
        }
        _ => {
            let _ = it.next();
        }
    }
}

pub fn clean(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            skip_ansi_sequence(&mut it);
            continue;
        }

        if !c.is_control()
            && !matches!(
                c,
                '\u{00AD}'
            | '\u{061C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{2028}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
            )
        {
            out.push(c);
        }
    }
    out
}

/// Columnas que ocupa un carácter en la terminal: 0 (combinantes), 1 o 2 (CJK, emojis).
fn char_width(c: char) -> usize {
    match c as u32 {
        0x0300..=0x036F
        | 0x1AB0..=0x1AFF
        | 0x1DC0..=0x1DFF
        | 0x20D0..=0x20FF
        | 0xFE00..=0xFE0F
        | 0xFE20..=0xFE2F => 0,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F680..=0x1F6FF
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

fn scan(s: &str, max: usize) -> (String, usize) {
    let (mut out, mut n) = (String::with_capacity(s.len()), 0);
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            skip_ansi_sequence(&mut it);
        } else if !c.is_control() {
            let w = char_width(c);
            if n + w > max {
                break;
            }
            out.push(c);
            n += w;
        }
    }
    (out, n)
}

pub fn visible_len(s: &str) -> usize {
    scan(s, usize::MAX).1
}

/// Recorta a `max` columnas conservando los códigos de color y estilo (SGR);
/// cualquier otra secuencia de escape se descarta.
fn truncate_keep_sgr(s: &str, max: usize) -> String {
    let (mut out, mut n) = (String::with_capacity(s.len()), 0);
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            if it.next_if_eq(&'[').is_some() {
                let mut params = String::new();
                let mut last = None;
                for ch in it.by_ref() {
                    if ch.is_ascii() && (b'@'..=b'~').contains(&(ch as u8)) {
                        last = Some(ch);
                        break;
                    }
                    params.push(ch);
                }
                if last == Some('m') && params.bytes().all(|b| b.is_ascii_digit() || b == b';') {
                    out.push_str("\x1b[");
                    out.push_str(&params);
                    out.push('m');
                }
            } else {
                skip_ansi_sequence(&mut it);
            }
        } else if !c.is_control() {
            let w = char_width(c);
            if n + w > max {
                break;
            }
            out.push(c);
            n += w;
        }
    }
    out
}

pub fn fit_width(s: &str, max: usize) -> String {
    let mut out = if visible_len(s) <= max {
        s.to_string()
    } else {
        truncate_keep_sgr(s, max)
    };
    out.push_str(reset());
    out
}

pub fn trunc(s: &str, max: usize) -> String {
    if visible_len(s) <= max {
        return s.to_string();
    }
    let (mut out, mut n) = (String::new(), 0);
    for c in s.chars() {
        let w = char_width(c);
        if n + w + 1 > max {
            break;
        }
        out.push(c);
        n += w;
    }
    out.push('…');
    out
}

/// Rellena con espacios a la derecha hasta `width` columnas (cuenta columnas, no caracteres).
pub fn pad(s: &str, width: usize) -> String {
    format!("{s}{}", " ".repeat(width.saturating_sub(visible_len(s))))
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

#[cfg(test)]
mod tests {
    use super::{bar, clean, fit_width, human, pad, reset, rgb_to_256, spark, trunc, visible_len};

    #[test]
    fn clean_removes_invisible_and_bidi_characters() {
        assert_eq!(
            clean("a\u{202E}b\u{200B}c\u{FEFF}d\u{2028}e\u{061C}f"),
            "abcdef"
        );
        assert_eq!(clean("ñandú — ok"), "ñandú — ok");
    }

    #[test]
    fn human_formats_binary_units() {
        assert_eq!(human(0, 1), "0 B");
        assert_eq!(human(1024, 1), "1.0 KiB");
        assert_eq!(human(1536, 1), "1.5 KiB");
        assert_eq!(human(5 * 1024 * 1024 * 1024, 1), "5.0 GiB");
    }

    #[test]
    fn trunc_shortens_with_an_ellipsis() {
        assert_eq!(trunc("abc", 3), "abc");
        assert_eq!(trunc("hello", 3), "he…");
    }

    #[test]
    fn bar_and_spark_clamp_out_of_range_values() {
        assert_eq!(visible_len(&bar(f64::NAN, 4)), 4);
        assert_eq!(visible_len(&bar(2.0, 4)), 4);
        assert_eq!(visible_len(&bar(-1.0, 4)), 4);
        assert_eq!(visible_len(&spark(&[0.0, 0.5, 1.0, 9.0], 8)), 8);
    }

    #[test]
    fn clean_removes_csi_and_osc_sequences() {
        let input = "before\x1b[31mred\x1b[0m\x1b]0;terminal title\x07after";
        assert_eq!(clean(input), "beforeredafter");
    }

    #[test]
    fn clean_removes_string_terminator_osc_and_control_characters() {
        let input = "a\x1b]8;;https://example.test\x1b\\linked\x1b]8;;\x1b\\b\n";
        assert_eq!(clean(input), "alinkedb");
    }

    #[test]
    fn visible_len_ignores_escape_sequences() {
        assert_eq!(visible_len("\x1b[1mhello\x1b[0m"), 5);
    }

    #[test]
    fn fit_width_keeps_color_when_the_line_fits() {
        let out = fit_width("\x1b[31mrojo\x1b[0m", 10);
        assert!(out.contains("\x1b[31m"));
        assert_eq!(visible_len(&out), 4);
    }

    #[test]
    fn wide_characters_count_as_two_columns() {
        assert_eq!(visible_len("日本語"), 6);
        assert_eq!(visible_len("a😀b"), 4);
        assert_eq!(visible_len("e\u{301}"), 1);
        assert_eq!(visible_len(&fit_width("日本語", 5)), 4);
        assert_eq!(trunc("日本語日本語", 5), "日本…");
        assert_eq!(pad("日本", 6), "日本  ");
    }

    #[test]
    fn rgb_maps_to_the_256_color_cube() {
        assert_eq!(rgb_to_256(0, 0, 0), 16);
        assert_eq!(rgb_to_256(255, 255, 255), 231);
        assert_eq!(rgb_to_256(255, 0, 0), 196);
        assert_eq!(rgb_to_256(0, 255, 0), 46);
    }

    #[test]
    fn fit_width_keeps_color_when_it_has_to_truncate() {
        let out = fit_width("\x1b[1m\x1b[34mGPU\x1b[0m  AMD Radeon Vega", 10);
        assert!(out.starts_with("\x1b[1m\x1b[34mGPU\x1b[0m"));
        assert_eq!(visible_len(&out), 10);
        assert!(out.ends_with(reset()));
        let out = fit_width("\x1b]0;titulo\x07\x1b[2Jabcdef", 3);
        assert_eq!(out, format!("abc{}", reset()));
    }

    #[test]
    fn fit_width_truncates_to_the_visible_width() {
        assert_eq!(visible_len(&fit_width("\x1b[31mabcdef\x1b[0m", 3)), 3);
    }
}
