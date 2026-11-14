mod anim;
mod disks;
mod info;
mod logo;
mod monitor;
mod ui;

use info::Info;
use monitor::Row;
use std::io::{self, IsTerminal};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, iter, process::ExitCode, thread};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use ui::{bold, dim, fit_width, reset, visible_len};

const GAP: usize = 4;
const DISK_TIMEOUT: Duration = Duration::from_secs(3);
const MIN_COLS: usize = 40;
const MIN_ROWS: usize = 12;

const HELP: &str = "rsi — información del sistema y monitor en vivo
USO: rsi [OPCIONES]
--once       imprime una vez y sale
--no-anim    logo estático
-h, --help   muestra esta ayuda

Por defecto permanece abierto (Enter, Esc o q para salir).";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layout {
    Side,
    Compact,
    TooSmall,
}

fn choose_layout(cols: usize, rows: usize, lw: usize) -> Layout {
    if cols < MIN_COLS || rows < MIN_ROWS {
        Layout::TooSmall
    } else if cols.saturating_sub(1).saturating_sub(lw + GAP) >= 30 {
        Layout::Side
    } else {
        Layout::Compact
    }
}

fn too_small_rows(cols: usize, rows: usize) -> Vec<Row> {
    let msg = vec![
        format!("Terminal demasiado pequeña: {cols}x{rows}"),
        format!("Mínimo necesario: {MIN_COLS}x{MIN_ROWS}"),
        "Agranda la ventana para ver rsi.".to_string(),
    ];
    let pad = rows.saturating_sub(2) / 2;
    let mut v = vec![(String::new(), String::new()); pad];
    v.extend(
        center(&msg, cols.saturating_sub(1))
            .into_iter()
            .map(|l| (l, String::new())),
    );
    v
}

fn right_groups(info: &Info, accent: &str, title: &str, disks: Vec<String>) -> Vec<Vec<String>> {
    let (b, d, r) = (bold(), dim(), reset());
    let head = |t: &str| format!("{title}{b}{t}{r}");
    let mut groups = vec![vec![
        format!("{accent}{b}{}{r}@{accent}{b}{}{r}", info.user, info.host),
        format!(
            "{d}{}{r}",
            "─".repeat(info.user.chars().count() + info.host.chars().count() + 1)
        ),
    ]];
    for s in &info.sections {
        let mut v = vec![head(s.title)];
        v.extend(
            s.rows
                .iter()
                .map(|(k, val)| format!("{accent}{b}{k:<9}{r} {val}")),
        );
        groups.push(v);
    }
    groups.push(iter::once(head("Almacenamiento")).chain(disks).collect());
    groups
}

fn width(lines: &[String]) -> usize {
    lines.iter().map(|l| visible_len(l)).max().unwrap_or(0)
}

fn center(lines: &[String], w: usize) -> Vec<String> {
    let left = w.saturating_sub(width(lines)) / 2;
    lines
        .iter()
        .map(|l| {
            format!(
                "{}{l}{}",
                " ".repeat(left),
                " ".repeat(w.saturating_sub(left + visible_len(l)))
            )
        })
        .collect()
}

fn left_column(logo: Vec<String>, live: Vec<String>, palette: &[String]) -> (Vec<String>, usize) {
    let w = width(&logo).max(width(&live)).max(width(palette));
    let mut col = center(&logo, w);
    for part in [&live[..], palette] {
        if !part.is_empty() {
            col.push(" ".repeat(w));
            col.extend(center(part, w));
        }
    }
    (col, w)
}

fn compact_rows(live: &[String], right: &[String]) -> Vec<Row> {
    live.iter()
        .map(|line| (line.clone(), String::new()))
        .chain([(String::new(), String::new())])
        .chain(right.iter().map(|line| (line.clone(), String::new())))
        .collect()
}

fn compose(left: &[String], lw: usize, right: &[String], cols: usize) -> Vec<Row> {
    let rw = cols.saturating_sub(1).saturating_sub(lw + GAP);
    let blank = " ".repeat(lw);
    (0..left.len().max(right.len()))
        .map(|i| {
            let live = left.get(i).unwrap_or(&blank).clone();
            let fixed = match right.get(i) {
                Some(r) if !r.is_empty() => format!("{}{}", " ".repeat(GAP), fit_width(r, rw)),
                _ => String::new(),
            };
            (live, fixed)
        })
        .collect()
}

fn main() -> ExitCode {
    let (mut once, mut animate) = (false, true);
    for a in env::args_os().skip(1) {
        let Ok(arg) = a.into_string() else {
            eprintln!("Opción no válida para la terminal\n\n{HELP}");
            return ExitCode::from(2);
        };
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "--once" | "--no-live" => once = true,
            "--no-anim" => animate = false,
            o => {
                eprintln!("Opción desconocida: {}\n\n{HELP}", ui::clean(o));
                return ExitCode::from(2);
            }
        }
    }

    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(disks::collect());
    });

    let info = info::gather(&sys);
    let disk_lines = match rx.recv_timeout(DISK_TIMEOUT) {
        Ok(list) => disks::lines(&list),
        Err(_) => vec![disks::timeout_line()],
    };

    let lg = logo::for_distro(&info.distro_id, info.id_like.as_deref());
    let accent = anim::accent(lg.hue);
    let title = anim::accent(lg.hue + 180.0);
    let groups = right_groups(&info, &accent, &title, disk_lines);

    let right_spaced = groups.join(&String::new());
    let right_compact = groups.concat();
    let palette = ui::palette();
    let mut mon = monitor::Monitor::new(sys);
    let interactive = io::stdout().is_terminal() && !once;

    monitor::run(
        &mut mon,
        animate && interactive,
        interactive,
        |m, phase, cols, rows| {
            let live = m.lines(&accent);
            let (left, lw) = left_column(anim::frame(&lg, phase), live.clone(), &palette);
            match choose_layout(cols, rows, lw) {
                Layout::TooSmall => too_small_rows(cols, rows),
                Layout::Compact => compact_rows(&live, &right_compact),
                Layout::Side => {
                    let total: usize = groups.iter().map(Vec::len).sum();
                    let spaced_h = (total + groups.len() - 1).max(left.len());
                    let right = if rows >= spaced_h + 2 {
                        &right_spaced
                    } else {
                        &right_compact
                    };
                    compose(&left, lw, right, cols)
                }
            }
        },
    );

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_pads_lines_to_the_requested_width() {
        assert_eq!(center(&["ab".to_string()], 6), vec!["  ab  ".to_string()]);
    }

    #[test]
    fn compose_places_columns_side_by_side_on_wide_terminals() {
        let rows = compose(&["L".to_string()], 1, &["R".to_string()], 80);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "L");
        assert_eq!(ui::clean(&rows[0].1), " ".repeat(GAP) + "R");
    }

    #[test]
    fn choose_layout_prefers_side_when_the_logo_fits_beside_the_info() {
        assert_eq!(choose_layout(120, 40, 41), Layout::Side);
        assert_eq!(choose_layout(80, 24, 41), Layout::Side);
    }

    #[test]
    fn choose_layout_uses_compact_when_there_is_no_room_for_the_logo() {
        assert_eq!(choose_layout(61, 45, 41), Layout::Compact);
    }

    #[test]
    fn choose_layout_reports_too_small_below_the_floor() {
        assert_eq!(choose_layout(39, 30, 41), Layout::TooSmall);
        assert_eq!(choose_layout(60, 11, 41), Layout::TooSmall);
    }
}
