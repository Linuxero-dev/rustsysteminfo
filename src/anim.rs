use crate::logo::Logo;
use crate::ui::{reset, rgb};

fn hsv(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let q = |f: f64| ((f + m) * 255.0).round() as u8;
    (q(r), q(g), q(b))
}

pub fn accent(hue: f64) -> String {
    let (r, g, b) = hsv(hue, 0.60, 1.0);
    rgb(r, g, b)
}

pub fn width(logo: &Logo) -> usize {
    logo.lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
}

pub fn frame(logo: &Logo, phase: f64) -> Vec<String> {
    let w = width(logo);
    logo.lines
        .iter()
        .enumerate()
        .map(|(y, line)| {
            let (mut s, mut last) = (String::new(), String::new());
            for (x, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let wave = (x as f64 * 0.35 + y as f64 * 0.50 - phase).sin();
                    let (r, g, b) = hsv(logo.hue + wave * 35.0, 0.65, 0.80 + 0.20 * wave);
                    let c = rgb(r, g, b);
                    if c != last {
                        s += &c;
                        last = c;
                    }
                }
                s.push(ch);
            }
            s + reset() + &" ".repeat(w.saturating_sub(line.chars().count()))
        })
        .collect()
}
