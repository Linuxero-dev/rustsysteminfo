use crate::logo::Logo;
use crate::ui::{reset, rgb};
use std::cell::RefCell;
use std::rc::Rc;

const LEVELS: usize = 16;

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
    let (r, g, b) = hsv(hue, 0.70, 1.0);
    rgb(r, g, b)
}

pub fn width(logo: &Logo) -> usize {
    logo.lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
}

fn density(c: char) -> f64 {
    match c {
        '.' | ',' | '\'' | '`' | ':' | ';' | '-' | '_' => 0.55,
        '/' | '\\' | '|' | '(' | ')' | '[' | ']' | '+' | '=' | '~' | '^' | '"' => 0.78,
        _ => 1.0,
    }
}

fn index(v: f64) -> usize {
    let n = (v.clamp(0.0, 1.0) * (LEVELS - 1) as f64).round() as usize;
    n.min(LEVELS - 1)
}

fn transitions(lut: &[String]) -> usize {
    lut.windows(2).filter(|w| w[0] != w[1]).count() + 1
}

fn levels(hue: f64) -> Vec<String> {
    let ramp = |h: f64, v: f64| {
        let (r, g, b) = hsv(h, 0.72, v);
        rgb(r, g, b)
    };
    let lut: Vec<String> = (0..LEVELS)
        .map(|i| ramp(hue, 0.30 + 0.70 * i as f64 / (LEVELS - 1) as f64))
        .collect();
    if transitions(&lut) >= 4 {
        return lut;
    }
    (0..LEVELS)
        .map(|i| {
            let t = i as f64 / (LEVELS - 1) as f64;
            ramp(hue + (t - 0.5) * 60.0, 0.85)
        })
        .collect()
}

thread_local! {
    static LUT: RefCell<Option<(u64, Rc<Vec<String>>)>> = const { RefCell::new(None) };
}

/// Tabla de colores del logo: se calcula una vez por tono y se reutiliza en cada fotograma.
fn cached_levels(hue: f64) -> Rc<Vec<String>> {
    LUT.with(|c| {
        let mut slot = c.borrow_mut();
        if let Some((bits, lut)) = slot.as_ref() {
            if *bits == hue.to_bits() {
                return Rc::clone(lut);
            }
        }
        let lut = Rc::new(levels(hue));
        *slot = Some((hue.to_bits(), Rc::clone(&lut)));
        lut
    })
}

pub fn frame(logo: &Logo, phase: f64) -> Vec<String> {
    let w = width(logo);
    let lut = cached_levels(logo.hue);
    logo.lines
        .iter()
        .enumerate()
        .map(|(y, line)| {
            let mut s = String::with_capacity(line.len() * 3 + 16);
            let mut last = usize::MAX;
            for (x, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let wave = (x as f64 * 0.30 + y as f64 * 0.45 - phase).sin();
                    let glint = wave.max(0.0).powi(8);
                    let v = (0.60 + 0.20 * wave) * density(ch) + 0.40 * glint;
                    let i = index(v);
                    if i != last {
                        s.push_str(&lut[i]);
                        last = i;
                    }
                }
                s.push(ch);
            }
            s.push_str(reset());
            for _ in line.chars().count()..w {
                s.push(' ');
            }
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{clean, visible_len};

    const ART: &[&str] = &["  /\\  ", " /  \\ ", "/____\\", "x"];

    fn logo() -> Logo {
        Logo {
            lines: ART,
            hue: 200.0,
        }
    }

    #[test]
    fn width_uses_the_longest_line() {
        assert_eq!(width(&logo()), 6);
        assert_eq!(
            width(&Logo {
                lines: &[],
                hue: 0.0
            }),
            0
        );
    }

    #[test]
    fn every_frame_line_is_padded_to_the_same_width() {
        for line in frame(&logo(), 1.3) {
            assert_eq!(visible_len(&line), 6);
        }
    }

    #[test]
    fn the_shape_of_the_logo_is_preserved() {
        for (drawn, original) in frame(&logo(), 2.0).iter().zip(ART) {
            assert_eq!(clean(drawn).trim_end(), original.trim_end());
        }
    }

    #[test]
    fn the_same_phase_produces_the_same_frame() {
        assert_eq!(frame(&logo(), 0.7), frame(&logo(), 0.7));
    }

    #[test]
    fn heavier_glyphs_are_brighter_than_light_ones() {
        assert!(density('.') < density('/'));
        assert!(density('/') < density('M'));
        assert_eq!(density('#'), 1.0);
    }

    #[test]
    fn the_level_index_never_leaves_the_table() {
        assert_eq!(index(-1.0), 0);
        assert_eq!(index(0.0), 0);
        assert_eq!(index(2.0), LEVELS - 1);
        assert_eq!(index(f64::NAN), 0);
        assert!(index(0.5) < LEVELS);
    }

    #[test]
    fn the_color_table_has_one_entry_per_level() {
        let lut = levels(200.0);
        assert_eq!(lut.len(), LEVELS);
        assert!((1..=LEVELS).contains(&transitions(&lut)));
    }

    #[test]
    fn the_color_table_is_reused_for_the_same_hue() {
        let a = cached_levels(120.0);
        let b = cached_levels(120.0);
        assert!(Rc::ptr_eq(&a, &b));
        let c = cached_levels(300.0);
        assert!(!Rc::ptr_eq(&a, &c));
    }

    #[test]
    fn an_empty_logo_draws_nothing() {
        assert!(frame(
            &Logo {
                lines: &[],
                hue: 0.0
            },
            0.0
        )
        .is_empty());
    }
}
