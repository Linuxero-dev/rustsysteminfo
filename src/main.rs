mod anim;
mod cache;
mod cli;
mod completions;
mod disks;
mod info;
mod json;
mod logo;
mod monitor;
mod pkgver;
mod proctree;
mod report;
mod ui;

use cli::Action;
use info::Info;
use monitor::Row;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;
use std::{env, iter, process::ExitCode, sync::mpsc, thread};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use ui::{bold, dim, fit_width, reset, visible_len};

const GAP: usize = 4;
const DISK_TIMEOUT: Duration = Duration::from_secs(3);
const MIN_COLS: usize = 40;
const MIN_ROWS: usize = 12;
/// Ancho de la línea decorativa que acompaña a los títulos de sección.
const HEAD_RULE: usize = 26;

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
    let head = |t: &str| {
        format!(
            "{title}{b}{t}{r} {d}{}{r}",
            "─".repeat(HEAD_RULE.saturating_sub(visible_len(t) + 1))
        )
    };
    let mut groups = vec![vec![
        format!("{accent}{b}{}{r}@{accent}{b}{}{r}", info.user, info.host),
        format!(
            "{d}{}{r}",
            "─".repeat(visible_len(&info.user) + visible_len(&info.host) + 1)
        ),
    ]];
    for s in &info.sections {
        let mut v = vec![head(s.title)];
        v.extend(
            s.rows
                .iter()
                .map(|(_, k, val)| format!("{accent}{b}{k:<9}{r} {val}")),
        );
        groups.push(v);
    }
    if !disks.is_empty() {
        groups.push(iter::once(head("Almacenamiento")).chain(disks).collect());
    }
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
            if !col.is_empty() {
                col.push(" ".repeat(w));
            }
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

fn print_text(text: &str) {
    let mut o = io::stdout().lock();
    let _ = o.write_all(text.as_bytes());
}

fn print_lines(lines: &[&str]) {
    let mut o = io::stdout().lock();
    for l in lines {
        if writeln!(o, "{l}").is_err() {
            break;
        }
    }
}

fn main() -> ExitCode {
    let mut args = Vec::new();
    for a in env::args_os().skip(1) {
        match a.into_string() {
            Ok(s) => args.push(s),
            Err(_) => {
                eprintln!("Opción no válida para la terminal\n\n{}", cli::HELP);
                return ExitCode::from(2);
            }
        }
    }
    let source = cli::config_source(&args);
    let (base, warnings) = match cli::load_config(&source, &info::cfg("rsi/config")) {
        Ok(loaded) => loaded,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let cli = match cli::parse_args(args, base) {
        Ok(Action::Run(cli)) => cli,
        Ok(Action::Help) => {
            print_lines(&[cli::HELP]);
            return ExitCode::SUCCESS;
        }
        Ok(Action::Version) => {
            print_lines(&[&format!("rsi {}", env!("CARGO_PKG_VERSION"))]);
            return ExitCode::SUCCESS;
        }
        Ok(Action::ListLogos) => {
            print_lines(logo::NAMES);
            return ExitCode::SUCCESS;
        }
        Ok(Action::ListModules) => {
            print_lines(info::MODULES);
            return ExitCode::SUCCESS;
        }
        Ok(Action::GenConfig) => {
            print_text(cli::SAMPLE_CONFIG);
            return ExitCode::SUCCESS;
        }
        Ok(Action::Report) => {
            let sys = System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            );
            let hidden: Vec<String> = ["ip", "host", "disks"].map(String::from).to_vec();
            let mut store = cache::Cache::load(false);
            let info = info::gather(&sys, &hidden, &mut store);
            print_text(&report::render(&info));
            return ExitCode::SUCCESS;
        }
        Ok(Action::Completions(shell)) => {
            print_text(&completions::script(&shell).unwrap_or_default());
            return ExitCode::SUCCESS;
        }
        Err(msg) => {
            eprintln!("{msg}\n\n{}", cli::HELP);
            return ExitCode::from(2);
        }
    };
    for w in &warnings {
        eprintln!("rsi: configuración: {w}");
    }

    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    let disk_rx = (!info::is_hidden(&cli.hidden, "disks")).then(|| {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(disks::collect());
        });
        rx
    });

    let mut store = cache::Cache::load(!cli.no_cache && !running_as_root());
    let info = info::gather(&sys, &cli.hidden, &mut store);
    store.save();
    // None = módulo oculto; Some(None) = tiempo agotado; Some(Some(lista)) = datos.
    let disk_list = disk_rx.map(|rx| rx.recv_timeout(DISK_TIMEOUT).ok());
    let mut mon = monitor::Monitor::new(sys);

    if cli.json {
        let snap = mon.measure(&mut store);
        store.save();
        let shown = disk_list.as_ref().and_then(|d| d.as_deref());
        print_text(&json::render(
            env!("CARGO_PKG_VERSION"),
            &info,
            shown,
            &snap,
        ));
        return ExitCode::SUCCESS;
    }

    let disk_lines = match &disk_list {
        None => Vec::new(),
        Some(None) => vec![disks::timeout_line()],
        Some(Some(list)) => disks::lines(list),
    };

    let lg = cli
        .logo
        .as_deref()
        .and_then(logo::by_name)
        .unwrap_or_else(|| logo::for_distro(&info.distro_id, info.id_like.as_deref()));
    let accent = anim::accent(lg.hue);
    let title = anim::accent(lg.hue + 180.0);
    let groups = right_groups(&info, &accent, &title, disk_lines);

    let right_spaced = groups.join(&String::new());
    let right_compact = groups.concat();
    let palette = if info::is_hidden(&cli.hidden, "colors") {
        Vec::new()
    } else {
        ui::palette()
    };
    let interactive = io::stdout().is_terminal() && !cli.once;

    monitor::run(
        &mut mon,
        cli.animate && interactive,
        interactive,
        &mut store,
        |m, phase, cols, rows| {
            let live = m.lines(&accent);
            let logo_lines = if cli.no_logo {
                Vec::new()
            } else {
                anim::frame(&lg, phase)
            };
            let (left, lw) = left_column(logo_lines, live.clone(), &palette);
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
    store.save();

    ExitCode::SUCCESS
}

/// Con sudo o como root no se usa la caché: dejaría ficheros de root en el HOME del usuario.
fn running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_column_without_logo_starts_with_the_meters() {
        let (col, _) = left_column(Vec::new(), vec!["CPU".to_string()], &[]);
        assert_eq!(col[0].trim(), "CPU");
    }

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
