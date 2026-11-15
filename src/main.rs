mod anim;
mod disks;
mod info;
mod logo;
mod monitor;
mod ui;

use info::Info;
use monitor::Row;
use std::io::{self, IsTerminal, Write};
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

General:
  -h, --help          muestra esta ayuda
  -V, --version       muestra la versión
      --once          imprime una vez y sale (sin monitor en vivo)
      --no-anim       logo estático

Logo:
      --logo NOMBRE   fuerza un logo (ver --list-logos)
      --no-logo       no muestra el logo
      --list-logos    lista los logos disponibles

Módulos:
      --no-MÓDULO     oculta un módulo, p. ej. --no-ip o --no-gpu
      --hide A,B      oculta varios módulos a la vez
      --list-modules  lista los módulos disponibles

Por defecto permanece abierto (Enter, Esc o q para salir).
Respeta la variable NO_COLOR.";

#[derive(Debug, PartialEq)]
struct Cli {
    once: bool,
    animate: bool,
    no_logo: bool,
    logo: Option<String>,
    hidden: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Action {
    Run(Cli),
    Help,
    Version,
    ListLogos,
    ListModules,
}

fn hide(hidden: &mut Vec<String>, module: &str) -> Result<(), String> {
    let m = module.to_ascii_lowercase();
    if !info::MODULES.contains(&m.as_str()) {
        return Err(format!(
            "Módulo desconocido: {} (ver --list-modules)",
            ui::clean(module)
        ));
    }
    if !hidden.contains(&m) {
        hidden.push(m);
    }
    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Action, String> {
    let mut cli = Cli {
        once: false,
        animate: true,
        no_logo: false,
        logo: None,
        hidden: Vec::new(),
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n, Some(v.to_string())),
            _ => (arg.as_str(), None),
        };
        let takes_value = matches!(name, "--logo" | "--hide");
        if inline.is_some() && !takes_value {
            return Err(format!("Opción desconocida: {}", ui::clean(&arg)));
        }
        match name {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--list-logos" => return Ok(Action::ListLogos),
            "--list-modules" => return Ok(Action::ListModules),
            "--once" | "--no-live" => cli.once = true,
            "--no-anim" | "--static" => cli.animate = false,
            "--no-logo" => cli.no_logo = true,
            "--logo" => {
                let v = inline
                    .or_else(|| it.next())
                    .ok_or("--logo necesita un nombre (ver --list-logos)")?;
                if logo::by_name(&v).is_none() {
                    return Err(format!(
                        "Logo desconocido: {} (ver --list-logos)",
                        ui::clean(&v)
                    ));
                }
                cli.logo = Some(v);
            }
            "--hide" => {
                let v = inline
                    .or_else(|| it.next())
                    .ok_or("--hide necesita una lista de módulos (ver --list-modules)")?;
                for m in v.split(',').filter(|m| !m.is_empty()) {
                    hide(&mut cli.hidden, m)?;
                }
            }
            o => match o.strip_prefix("--no-") {
                Some(m) => hide(&mut cli.hidden, m)?,
                None => return Err(format!("Opción desconocida: {}", ui::clean(o))),
            },
        }
    }
    Ok(Action::Run(cli))
}

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
                eprintln!("Opción no válida para la terminal\n\n{HELP}");
                return ExitCode::from(2);
            }
        }
    }
    let cli = match parse_args(args) {
        Ok(Action::Run(cli)) => cli,
        Ok(Action::Help) => {
            print_lines(&[HELP]);
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
        Err(msg) => {
            eprintln!("{msg}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };

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

    let info = info::gather(&sys, &cli.hidden);
    let disk_lines = match disk_rx {
        None => Vec::new(),
        Some(rx) => match rx.recv_timeout(DISK_TIMEOUT) {
            Ok(list) => disks::lines(&list),
            Err(_) => vec![disks::timeout_line()],
        },
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
    let mut mon = monitor::Monitor::new(sys);
    let interactive = io::stdout().is_terminal() && !cli.once;

    monitor::run(
        &mut mon,
        cli.animate && interactive,
        interactive,
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

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn run_cli(a: &[&str]) -> Cli {
        match parse_args(args(a)) {
            Ok(Action::Run(c)) => c,
            other => panic!("se esperaba Run, salió {other:?}"),
        }
    }

    #[test]
    fn parse_defaults_run_the_live_monitor() {
        let c = run_cli(&[]);
        assert!(c.animate && !c.once && !c.no_logo);
        assert!(c.logo.is_none() && c.hidden.is_empty());
    }

    #[test]
    fn parse_simple_actions() {
        assert_eq!(parse_args(args(&["-h"])), Ok(Action::Help));
        assert_eq!(parse_args(args(&["--version"])), Ok(Action::Version));
        assert_eq!(parse_args(args(&["--list-logos"])), Ok(Action::ListLogos));
        assert_eq!(
            parse_args(args(&["--list-modules"])),
            Ok(Action::ListModules)
        );
    }

    #[test]
    fn parse_hides_modules_with_no_prefix_and_hide_list() {
        assert_eq!(run_cli(&["--no-ip"]).hidden, vec!["ip"]);
        assert_eq!(
            run_cli(&["--no-gpu", "--hide", "IP,disks", "--hide=ip"]).hidden,
            vec!["gpu", "ip", "disks"]
        );
    }

    #[test]
    fn parse_keeps_the_flags_that_start_with_no() {
        let c = run_cli(&["--no-anim", "--no-logo", "--no-live"]);
        assert!(!c.animate && c.no_logo && c.once);
        assert!(c.hidden.is_empty());
    }

    #[test]
    fn parse_logo_accepts_known_names_only() {
        assert_eq!(run_cli(&["--logo", "arch"]).logo.as_deref(), Some("arch"));
        assert_eq!(run_cli(&["--logo=tux"]).logo.as_deref(), Some("tux"));
        assert!(parse_args(args(&["--logo", "no-existe"])).is_err());
        assert!(parse_args(args(&["--logo"])).is_err());
    }

    #[test]
    fn parse_rejects_unknown_options_and_modules() {
        assert!(parse_args(args(&["--nope"])).is_err());
        assert!(parse_args(args(&["--no-nada"])).is_err());
        assert!(parse_args(args(&["--hide", "ip,nada"])).is_err());
        assert!(parse_args(args(&["--once=1"])).is_err());
        assert!(parse_args(args(&["--hide"])).is_err());
    }

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
