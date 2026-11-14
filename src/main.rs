mod anim;
mod disks;
mod info;
mod logo;
mod monitor;
mod ui;

use info::Info;
use monitor::Row;
use std::io::{self, IsTerminal};
use std::{env, iter, process::ExitCode, thread};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use ui::{bold, dim, fit_width, reset, visible_len};

const GAP: usize = 4;
const HELP: &str = "rsi — información del sistema y monitor en vivo
USO: rsi [OPCIONES]
--once       imprime una vez y sale
--no-anim    logo estático
-h, --help   muestra esta ayuda

Por defecto permanece abierto (Enter, Esc o q para salir).";

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

fn compose(left: &[String], lw: usize, right: &[String], cols: usize) -> Vec<Row> {
    let rw = cols.saturating_sub(1).saturating_sub(lw + GAP);
    if rw < 30 {
        return left
            .iter()
            .map(|l| (l.clone(), String::new()))
            .chain([(String::new(), String::new())])
            .chain(right.iter().map(|r| (String::new(), r.clone())))
            .collect();
    }
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
    for a in env::args().skip(1) {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "--once" | "--no-live" => once = true,
            "--no-anim" => animate = false,
            o => {
                eprintln!("Opción desconocida: {o}\n\n{HELP}");
                return ExitCode::from(2);
            }
        }
    }

    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    let (info, disk_list) = thread::scope(|s| {
        let h = s.spawn(disks::collect);
        (info::gather(&sys), h.join().unwrap_or_default())
    });

    let lg = logo::for_distro(&info.distro_id, info.id_like.as_deref());
    let accent = anim::accent(lg.hue);
    let title = anim::accent(lg.hue + 180.0);
    let groups = right_groups(&info, &accent, &title, disks::lines(&disk_list));
    let palette = ui::palette();
    let mut mon = monitor::Monitor::new(sys);
    let interactive = io::stdout().is_terminal() && !once;

    monitor::run(
        &mut mon,
        animate && interactive,
        interactive,
        |m, phase, cols, rows| {
            let (left, lw) = left_column(anim::frame(&lg, phase), m.lines(&accent), &palette);
            let total: usize = groups.iter().map(Vec::len).sum();
            let spaced = rows >= (total + groups.len() - 1).max(left.len()) + 2;
            let right = if spaced {
                groups.join(&String::new())
            } else {
                groups.concat()
            };
            compose(&left, lw, &right, cols)
        },
    );

    ExitCode::SUCCESS
}
