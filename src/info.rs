use crate::cache::{self, Cache};
use crate::ui::{clean, human};
use crate::{drm, pkgver, proctree};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{env, fs, thread};
use sysinfo::System;

const TRUSTED_BIN_DIRS: &[&str] = &[
    "/usr/bin",
    "/bin",
    "/usr/sbin",
    "/sbin",
    "/usr/local/bin",
    "/run/current-system/sw/bin",
    "/run/wrappers/bin",
];

const MAX_OUTPUT: u64 = 16 * 1024 * 1024;
/// Plazo para consultar bases de paquetes grandes (rpm, Nix) en discos lentos.
const SLOW_DB: Duration = Duration::from_secs(8);

/// Identificadores de módulo que se pueden ocultar con `--no-<módulo>` o `--hide`.
pub const MODULES: &[&str] = &[
    "os",
    "host",
    "kernel",
    "uptime",
    "init",
    "processes",
    "load",
    "packages",
    "shell",
    "terminal",
    "locale",
    "ip",
    "de",
    "wm",
    "display",
    "theme",
    "icons",
    "font",
    "cpu",
    "temp",
    "gpu",
    "battery",
    "disks",
    "colors",
];

pub fn is_hidden(hidden: &[String], id: &str) -> bool {
    hidden.iter().any(|h| h == id)
}

pub struct Section {
    pub title: &'static str,
    /// (identificador de módulo, etiqueta, valor)
    pub rows: Vec<(&'static str, &'static str, String)>,
}

pub struct Info {
    pub user: String,
    pub host: String,
    pub distro_id: String,
    pub id_like: Option<String>,
    pub sections: Vec<Section>,
}

type Theme = [Option<String>; 3];

pub(crate) fn var(k: &str) -> Option<String> {
    env::var(k).ok().filter(|v| !v.is_empty())
}

pub(crate) fn read(p: impl AsRef<Path>) -> Option<String> {
    let s = fs::read_to_string(p).ok()?;
    let s = s.trim_matches(|c: char| c.is_whitespace() || c == '\0');
    (!s.is_empty()).then(|| s.to_string())
}

fn find_bin(bin: &str) -> Option<PathBuf> {
    if let Some(p) = TRUSTED_BIN_DIRS
        .iter()
        .map(|d| Path::new(d).join(bin))
        .find(|p| p.is_file())
    {
        return Some(p);
    }

    if unsafe { libc::geteuid() } == 0 {
        None
    } else {
        Some(PathBuf::from(bin))
    }
}

pub(crate) fn run(bin: &str, args: &[&str]) -> Option<String> {
    run_with(bin, args, Duration::from_secs(2), true)
}

/// `limit` para bases de paquetes grandes; `need_ok = false` acepta códigos de
/// salida distintos de cero (p. ej. `rpm -q` con paquetes que no están).
fn run_with(bin: &str, args: &[&str], limit: Duration, need_ok: bool) -> Option<String> {
    let mut child = Command::new(find_bin(bin)?)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    let stdout = child.stdout.take()?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.take(MAX_OUTPUT).read_to_end(&mut buf);
        let _ = tx.send(buf);
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() <= limit => thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };

    let buf = rx.recv_timeout(Duration::from_millis(500)).ok()?;
    (status.success() || !need_ok).then(|| String::from_utf8_lossy(&buf).into_owned())
}

fn count_dir(path: &str) -> usize {
    fs::read_dir(path).map_or(0, Iterator::count)
}

fn count_lines(path: &str, pred: impl Fn(&str) -> bool) -> usize {
    fs::read(path).map_or(0, |b| {
        String::from_utf8_lossy(&b)
            .lines()
            .filter(|l| pred(l))
            .count()
    })
}

fn ini(path: &str, section: &str, key: &str) -> Option<String> {
    let txt = fs::read_to_string(path).ok()?;
    let mut cur = "";
    for l in txt.lines().map(str::trim) {
        if let Some(s) = l.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            cur = s;
        } else if cur == section {
            if let Some((k, v)) = l.split_once('=') {
                if k.trim() == key && !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

pub fn cfg(rel: &str) -> String {
    let base = var("XDG_CONFIG_HOME")
        .filter(|p| Path::new(p).is_absolute())
        .or_else(|| var("HOME").map(|h| format!("{h}/.config")));
    base.map_or_else(String::new, |b| format!("{b}/{rel}"))
}

type RawRow = (&'static str, &'static str, Option<String>);

fn section(title: &'static str, hidden: &[String], rows: Vec<RawRow>) -> Option<Section> {
    let rows: Vec<(&'static str, &'static str, String)> = rows
        .into_iter()
        .filter(|(id, _, _)| !is_hidden(hidden, id))
        .filter_map(|(id, k, v)| v.map(|val| (id, k, clean(&val))))
        .filter(|(_, _, v)| !v.is_empty())
        .collect();
    (!rows.is_empty()).then_some(Section { title, rows })
}

fn os_release() -> (Option<String>, Option<String>, Option<String>) {
    let txt = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    let get = |k: &str| {
        txt.lines()
            .find_map(|l| l.strip_prefix(k)?.strip_prefix('='))
            .map(|v| v.trim().trim_matches(&['"', '\''][..]).to_string())
            .filter(|v| !v.is_empty())
    };
    (get("ID"), get("ID_LIKE"), get("PRETTY_NAME"))
}

fn junk(s: &str) -> bool {
    let l = s.to_lowercase();
    [
        "o.e.m",
        "default string",
        "system product name",
        "system version",
        "not specified",
        "not applicable",
    ]
    .iter()
    .any(|j| l.contains(j))
        || matches!(l.as_str(), "none" | "unknown" | "n/a")
}

fn host_model() -> Option<String> {
    let dmi = |f: &str| read(format!("/sys/class/dmi/id/{f}")).filter(|s| !junk(s));
    let (vendor, product, version) = (
        dmi("sys_vendor"),
        dmi("product_name"),
        dmi("product_version"),
    );
    let mut s = match (vendor, product) {
        (Some(v), Some(p)) if p.to_lowercase().starts_with(&v.to_lowercase()) => p,
        (Some(v), Some(p)) => format!("{v} {p}"),
        (v, p) => v.or(p)?,
    };
    if let Some(ver) = version {
        if !s.to_lowercase().contains(&ver.to_lowercase()) {
            s = format!("{s} {ver}");
        }
    }
    Some(s)
}

fn uptime(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs % 86_400 / 3600, secs % 3600 / 60);
    [(d, "d"), (h, "h")]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, u)| format!("{n}{u}"))
        .chain([format!("{m}m")])
        .collect::<Vec<String>>()
        .join(" ")
}

fn processes() -> Option<String> {
    let n = fs::read_dir("/proc")
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
        })
        .count();
    (n > 0).then(|| n.to_string())
}

fn packages() -> Option<String> {
    let lines = |o: Option<String>| o.map_or(0, |s| s.lines().count());
    let home = var("HOME").unwrap_or_default();
    let has_rpm_db =
        Path::new("/var/lib/rpm").is_dir() || Path::new("/usr/lib/sysimage/rpm").is_dir();
    let portage: usize = fs::read_dir("/var/db/pkg").map_or(0, |d| {
        d.flatten()
            .map(|e| count_dir(&e.path().to_string_lossy()))
            .sum()
    });
    let snaps = fs::read_dir("/var/lib/snapd/snaps").map_or(0, |d| {
        d.flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".snap"))
            .count()
    });
    let nix = if Path::new("/run/current-system/sw").exists() {
        lines(run_with(
            "nix-store",
            &["-q", "--requisites", "/run/current-system/sw"],
            SLOW_DB,
            true,
        ))
    } else {
        0
    };
    let counts = [
        (
            "rpm",
            if has_rpm_db {
                lines(run_with("rpm", &["-qa", "--qf", "x\n"], SLOW_DB, true))
            } else {
                0
            },
        ),
        (
            "dpkg",
            count_lines("/var/lib/dpkg/status", |l| {
                l == "Status: install ok installed"
            }),
        ),
        (
            "pacman",
            count_dir("/var/lib/pacman/local").saturating_sub(1),
        ),
        (
            "apk",
            count_lines("/lib/apk/db/installed", |l| l.starts_with("P:")),
        ),
        (
            "xbps",
            if Path::new("/var/db/xbps").is_dir() {
                lines(run("xbps-query", &["-l"]))
            } else {
                0
            },
        ),
        ("portage", portage),
        ("nix", nix),
        ("pkgtools", count_dir("/var/log/packages")),
        (
            "flatpak",
            count_dir("/var/lib/flatpak/app")
                + count_dir(&format!("{home}/.local/share/flatpak/app")),
        ),
        ("snap", snaps),
    ];
    let v: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| format!("{n} ({k})"))
        .collect();
    (!v.is_empty()).then(|| v.join(", "))
}

/// Primer número de versión de una línea como `GNU bash, version 5.2.26(1)-release`.
fn parse_version(line: &str) -> Option<String> {
    let token = line
        .split_whitespace()
        .find(|t| t.starts_with(|c: char| c.is_ascii_digit()))?;
    let v: String = token
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let v = v.trim_end_matches('.');
    (!v.is_empty()).then(|| v.to_string())
}

fn shell() -> Option<String> {
    let s = var("SHELL")?;
    let name = Path::new(&s).file_name()?.to_string_lossy().into_owned();
    let version = matches!(name.as_str(), "bash" | "zsh" | "fish" | "nu")
        .then(|| run(&name, &["--version"]))
        .flatten()
        .and_then(|o| o.lines().next().and_then(parse_version));
    Some(match version {
        Some(v) => format!("{name} {v}"),
        None => name,
    })
}

fn terminal(versions: &HashMap<String, String>) -> Option<String> {
    if let Some(p) = var("TERM_PROGRAM") {
        return Some(match var("TERM_PROGRAM_VERSION") {
            Some(v) => format!("{p} {v}"),
            None => p,
        });
    }
    if let Some(n) = var("KONSOLE_VERSION").and_then(|v| v.parse::<u32>().ok()) {
        return Some(format!(
            "Konsole {}.{}.{}",
            n / 10000,
            n / 100 % 100,
            n % 100
        ));
    }
    const KNOWN: &[(&str, &str)] = &[
        ("KONSOLE_VERSION", "Konsole"),
        ("KITTY_WINDOW_ID", "kitty"),
        ("ALACRITTY_SOCKET", "Alacritty"),
        ("WEZTERM_PANE", "WezTerm"),
        ("GNOME_TERMINAL_SERVICE", "GNOME Terminal"),
        ("GNOME_TERMINAL_SCREEN", "GNOME Terminal"),
        ("TILIX_ID", "Tilix"),
        ("VTE_VERSION", "VTE"),
    ];
    if let Some((_, n)) = KNOWN.iter().find(|(k, _)| var(k).is_some()) {
        return Some(n.to_string());
    }
    if let Some(t) = proctree::find_terminal(Path::new("/proc"), std::process::id()) {
        let ver = t.2.iter().find_map(|p| versions.get(*p));
        return Some(match ver {
            Some(v) => format!("{} {v}", t.1),
            None => t.1.to_string(),
        });
    }
    var("TERM")
}

fn locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter().find_map(var)
}

fn desktop(versions: &HashMap<String, String>) -> Option<String> {
    let name = var("XDG_CURRENT_DESKTOP")
        .or_else(|| var("DESKTOP_SESSION"))
        .or_else(|| var("XDG_SESSION_DESKTOP"))?;
    let with_ver = |base: &str, bin: &str| match run(bin, &["--version"])
        .and_then(|o| o.split_whitespace().last().map(String::from))
    {
        Some(v) => format!("{base} {v}"),
        None => base.to_string(),
    };
    let label = if name.contains("KDE") {
        with_ver("KDE Plasma", "plasmashell")
    } else if name.contains("GNOME") {
        with_ver("GNOME", "gnome-shell")
    } else if name.to_ascii_uppercase().contains("COSMIC") {
        match versions.get("cosmic-session") {
            Some(v) => format!("COSMIC {v}"),
            None => "COSMIC".to_string(),
        }
    } else {
        name
    };
    let session = match var("XDG_SESSION_TYPE").as_deref() {
        Some("wayland") => " (Wayland)",
        Some("x11") => " (X11)",
        _ => "",
    };
    Some(format!("{label}{session}"))
}

/// Gestores de ventanas y compositores conocidos: (nombre del proceso, nombre mostrado).
const WMS: &[(&str, &str)] = &[
    ("kwin_wayland", "KWin"),
    ("kwin_x11", "KWin"),
    ("gnome-shell", "Mutter"),
    ("mutter", "Mutter"),
    ("Hyprland", "Hyprland"),
    ("hyprland", "Hyprland"),
    ("sway", "Sway"),
    ("niri", "niri"),
    ("river", "river"),
    ("labwc", "labwc"),
    ("wayfire", "Wayfire"),
    ("cosmic-comp", "COSMIC Comp"),
    ("weston", "Weston"),
    ("xfwm4", "Xfwm4"),
    ("marco", "Marco"),
    ("muffin", "Muffin"),
    ("budgie-wm", "Budgie WM"),
    ("openbox", "Openbox"),
    ("i3", "i3"),
    ("bspwm", "bspwm"),
    ("awesome", "awesome"),
    ("qtile", "Qtile"),
    ("herbstluftwm", "herbstluftwm"),
    ("icewm", "IceWM"),
    ("fluxbox", "Fluxbox"),
    ("enlightenment", "Enlightenment"),
    ("dwm", "dwm"),
];

fn find_wm(comms: &HashSet<String>) -> Option<&'static str> {
    WMS.iter()
        .find(|(comm, _)| comms.contains(*comm))
        .map(|(_, name)| *name)
}

fn wm() -> Option<String> {
    let comms: HashSet<String> = fs::read_dir("/proc")
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
        })
        .filter_map(|e| read(e.path().join("comm")))
        .collect();
    find_wm(&comms).map(String::from)
}

/// Frecuencia de refresco (Hz) del primer descriptor de temporización del EDID
/// cuya resolución activa coincide con `mode` (p. ej. `1920x1080`).
fn edid_refresh(edid: &[u8], mode: &str) -> Option<u32> {
    let (w, h) = mode.split_once('x')?;
    let w: u32 = w.parse().ok()?;
    let h: u32 = h
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    if edid.len() < 128 {
        return None;
    }
    [54usize, 72, 90, 108].iter().find_map(|&o| {
        let d = &edid[o..o + 18];
        let clock = u64::from(u16::from_le_bytes([d[0], d[1]])) * 10_000;
        let ha = u32::from(d[2]) | (u32::from(d[4] & 0xF0) << 4);
        let hb = u32::from(d[3]) | (u32::from(d[4] & 0x0F) << 8);
        let va = u32::from(d[5]) | (u32::from(d[7] & 0xF0) << 4);
        let vb = u32::from(d[6]) | (u32::from(d[7] & 0x0F) << 8);
        let total = u64::from(ha + hb) * u64::from(va + vb);
        if clock == 0 || total == 0 || ha != w || va != h {
            return None;
        }
        let hz = ((clock + total / 2) / total) as u32;
        (20..=500).contains(&hz).then_some(hz)
    })
}

/// Nombre del monitor (descriptor `0xFC`) y diagonal en pulgadas, del EDID.
/// El texto es ASCII imprimible: el EDID es una entrada no confiable.
fn edid_identity(edid: &[u8]) -> (Option<String>, Option<u32>) {
    if edid.len() < 128 {
        return (None, None);
    }
    let name = [54usize, 72, 90, 108].iter().find_map(|&o| {
        let d = &edid[o..o + 18];
        if d[..3] != [0, 0, 0] || d[3] != 0xFC {
            return None;
        }
        let t: String = d[5..]
            .iter()
            .take_while(|&&c| c != 0x0A)
            .filter(|c| (0x20..0x7F).contains(*c))
            .map(|&c| c as char)
            .collect();
        let t = t.trim();
        (!t.is_empty()).then(|| t.to_string())
    });
    // Bytes 21 y 22: tamaño en cm (si el segundo es 0, el par es una proporción, no un tamaño).
    let (w, h) = (f64::from(edid[21]), f64::from(edid[22]));
    let inches = (w > 0.0 && h > 0.0).then(|| (w.hypot(h) / 2.54).round() as u32);
    (name, inches.filter(|i| (5..=150).contains(i)))
}

fn displays() -> Option<String> {
    let mut v: Vec<String> = fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let (card, port) = name.split_once('-')?;
            let port = port.to_string();
            let p = e.path();
            if read(p.join("status"))? != "connected" {
                return None;
            }
            let edid = fs::read(p.join("edid")).unwrap_or_default();
            // Modo realmente activo (DRM); si no se puede, el preferido del EDID.
            let active = read(p.join("connector_id"))
                .and_then(|id| id.parse().ok())
                .and_then(|id| drm::active_mode(card, id));
            let mode = match active {
                Some(m) => format!("{}x{}", m.width, m.height),
                None => read(p.join("modes"))?.lines().next()?.to_string(),
            };
            let hz = active.map(|m| m.hz).or_else(|| edid_refresh(&edid, &mode));
            let (model, inches) = edid_identity(&edid);
            let mut line = match model {
                Some(m) => format!("{port} ({m}): {mode}"),
                None => format!("{port}: {mode}"),
            };
            if let Some(hz) = hz {
                line.push_str(&format!(" @ {hz} Hz"));
            }
            if let Some(i) = inches {
                line.push_str(&format!(", {i}\""));
            }
            Some(line)
        })
        .collect();
    v.sort();
    (!v.is_empty()).then(|| v.join(", "))
}

fn font(raw: &str) -> String {
    let mut it = raw.split(',').map(str::trim);
    match (it.next(), it.next()) {
        (Some(n), Some(s))
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.') =>
        {
            format!("{n} ({s}pt)")
        }
        (Some(n), _) => n.to_string(),
        _ => raw.to_string(),
    }
}

fn kde_theme() -> Theme {
    let f = cfg("kdeglobals");
    let theme = match (
        ini(&f, "KDE", "widgetStyle"),
        ini(&f, "General", "ColorScheme"),
    ) {
        (Some(a), Some(b)) => Some(format!("{a} / {b}")),
        (a, b) => a.or(b),
    };
    [
        theme,
        ini(&f, "Icons", "Theme"),
        ini(&f, "General", "font").map(|s| font(&s)),
    ]
}

fn gtk_theme() -> Theme {
    for v in ["3.0", "4.0"] {
        let f = cfg(&format!("gtk-{v}/settings.ini"));
        let t = [
            ini(&f, "Settings", "gtk-theme-name"),
            ini(&f, "Settings", "gtk-icon-theme-name"),
            ini(&f, "Settings", "gtk-font-name").map(|s| font(&s)),
        ];
        if t.iter().any(Option::is_some) {
            return t;
        }
    }
    Theme::default()
}

fn gnome_theme() -> Theme {
    let g = |k: &str| {
        run("gsettings", &["get", "org.gnome.desktop.interface", k])
            .map(|s| s.trim().trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
    };
    [
        g("gtk-theme"),
        g("icon-theme"),
        g("font-name").map(|s| font(&s)),
    ]
}

fn appearance() -> Theme {
    let de = var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let order: [fn() -> Theme; 3] = if de.contains("KDE") {
        [kde_theme, gtk_theme, gnome_theme]
    } else if de.contains("GNOME") {
        [gnome_theme, gtk_theme, kde_theme]
    } else {
        [gtk_theme, gnome_theme, kde_theme]
    };
    order
        .iter()
        .map(|f| f())
        .find(|t| t.iter().any(Option::is_some))
        .unwrap_or_default()
}

fn cpu(sys: &System) -> Option<String> {
    let first = sys.cpus().first()?;
    let mut name = first.brand().to_string();
    if name.trim().is_empty() {
        name = run("lscpu", &[])
            .and_then(|o| {
                o.lines()
                    .find_map(|l| l.strip_prefix("Model name:").map(|s| s.trim().to_string()))
            })
            .unwrap_or_else(|| "CPU".into());
    }
    for sep in [" @ ", " w/ ", " with "] {
        if let Some(i) = name.find(sep) {
            name.truncate(i);
        }
    }
    let name = ["(R)", "(TM)", "(tm)", "CPU", "Processor"]
        .into_iter()
        .fold(name, |n, p| n.replace(p, " "));
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let threads = sys.cpus().len();
    let topo = match sys.physical_core_count() {
        Some(p) => format!("{p}C/{threads}T"),
        None => format!("{threads}T"),
    };
    let mhz = read("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(first.frequency(), |k| k / 1000);
    Some(if mhz > 0 {
        format!("{name} ({topo}) @ {:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{name} ({topo})")
    })
}

fn lspci_gpus() -> Option<Vec<String>> {
    let v: Vec<String> = run("lspci", &["-mm"])?
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('"').collect();
            if f.len() < 6 || !["VGA", "3D", "Display"].iter().any(|k| f[1].contains(k)) {
                return None;
            }
            let vendor = ["Intel", "NVIDIA", "AMD"]
                .into_iter()
                .find(|v| f[3].contains(*v))
                .unwrap_or(f[3]);
            let dev = match (f[5].rfind('['), f[5].rfind(']')) {
                (Some(a), Some(b)) if a < b => &f[5][a + 1..b],
                _ => f[5],
            };
            Some(format!("{vendor} {dev}"))
        })
        .collect();
    (!v.is_empty()).then_some(v)
}

fn drm_gpus() -> Option<Vec<String>> {
    let v: Vec<String> = fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            if !n.starts_with("card") || n.contains('-') {
                return None;
            }
            let vendor = match read(e.path().join("device/vendor"))?.as_str() {
                "0x8086" => "Intel",
                "0x10de" => "NVIDIA",
                "0x1002" => "AMD",
                _ => return None,
            };
            Some(vendor.to_string())
        })
        .collect();
    (!v.is_empty()).then_some(v)
}

/// Temperatura de la CPU leída de `hwmon` (`root` es normalmente `/sys/class/hwmon`).
fn cpu_temp_in(root: &Path) -> Option<String> {
    const CHIPS: &[&str] = &["k10temp", "coretemp", "zenpower", "cpu_thermal", "acpitz"];
    const LABELS: &[&str] = &["Tctl", "Tdie", "Package id 0", "CPU"];
    let mut best: Option<(usize, i64)> = None;
    for e in fs::read_dir(root).ok()?.flatten() {
        let p = e.path();
        let Some(rank) = read(p.join("name")).and_then(|n| CHIPS.iter().position(|c| *c == n))
        else {
            continue;
        };
        let labelled = (1..=8).find_map(|i| {
            let label = read(p.join(format!("temp{i}_label")))?;
            if LABELS.contains(&label.as_str()) {
                read(p.join(format!("temp{i}_input")))
            } else {
                None
            }
        });
        let raw = labelled.or_else(|| read(p.join("temp1_input")));
        let Some(milli) = raw.and_then(|s| s.parse::<i64>().ok()) else {
            continue;
        };
        if !(1..=150_000).contains(&milli) {
            continue;
        }
        match best {
            Some((r, _)) if r <= rank => {}
            _ => best = Some((rank, milli)),
        }
    }
    best.map(|(_, milli)| format!("{} °C", (milli as f64 / 1000.0).round() as i64))
}

fn cpu_temp() -> Option<String> {
    cpu_temp_in(Path::new("/sys/class/hwmon"))
}

fn parse_ip(out: &str) -> Option<String> {
    const SKIP: &[&str] = &["docker", "br-", "veth", "virbr"];
    let v: Vec<String> = out
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let iface = it.nth(1)?;
            if SKIP.iter().any(|p| iface.starts_with(p)) {
                return None;
            }
            let addr = it.skip_while(|t| *t != "inet").nth(1)?;
            Some(format!("{iface}: {addr}"))
        })
        .collect();
    (!v.is_empty()).then(|| v.join(", "))
}

fn local_ip() -> Option<String> {
    parse_ip(&run(
        "ip",
        &["-o", "-4", "addr", "show", "scope", "global"],
    )?)
}

fn nvidia_vram() -> Option<u64> {
    let out = run(
        "nvidia-smi",
        &["--query-gpu=memory.total", "--format=csv,noheader,nounits"],
    )?;
    let mib: u64 = out.lines().next()?.trim().parse().ok()?;
    Some(mib * 1024 * 1024)
}

/// VRAM total: AMD la expone en sysfs y NVIDIA vía `nvidia-smi`.
fn gpu_vram(name: &str) -> Option<u64> {
    let amd = fs::read_dir("/sys/class/drm")
        .ok()
        .and_then(|d| {
            d.flatten()
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.starts_with("card") && !n.contains('-')
                })
                .find_map(|e| {
                    read(e.path().join("device/mem_info_vram_total"))?
                        .parse::<u64>()
                        .ok()
                })
        })
        .filter(|b| *b > 0);
    amd.or_else(|| name.contains("NVIDIA").then(nvidia_vram).flatten())
}

/// Driver de kernel de la primera GPU conocida (`amdgpu`, `i915`, `nouveau`,
/// `nvidia` con su versión…), leído de sysfs.
fn gpu_driver() -> Option<String> {
    let mut cards: Vec<PathBuf> = fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("card") && !n.contains('-')
        })
        .map(|e| e.path().join("device"))
        .collect();
    cards.sort();
    cards.iter().find_map(|dev| {
        let drv = fs::read_link(dev.join("driver")).ok()?;
        let drv = drv.file_name()?.to_string_lossy().into_owned();
        if !drv
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return None;
        }
        Some(match read("/sys/module/nvidia/version") {
            Some(v) if drv == "nvidia" => format!("nvidia {v}"),
            _ => drv,
        })
    })
}

fn gpus() -> Option<String> {
    let list = lspci_gpus().or_else(drm_gpus)?;
    let names = list.join(", ");
    // Con varias GPU no se sabe a cuál pertenece cada cifra: solo se muestra con una.
    let vram = if list.len() == 1 {
        gpu_vram(&list[0])
    } else {
        None
    };
    let driver = if list.len() == 1 { gpu_driver() } else { None };
    let mut out = match vram {
        Some(b) => format!("{names} ({})", human(b, 1)),
        None => names,
    };
    if let Some(d) = driver {
        out.push_str(&format!(" [{d}]"));
    }
    Some(out)
}

fn battery() -> Option<String> {
    fs::read_dir("/sys/class/power_supply")
        .ok()?
        .flatten()
        .find_map(|e| {
            let p = e.path();
            if read(p.join("type")).as_deref() != Some("Battery")
                || read(p.join("scope")).as_deref() == Some("Device")
            {
                return None;
            }
            let cap = read(p.join("capacity"))?;
            let st = read(p.join("status")).unwrap_or_default();
            let st = match st.as_str() {
                "Charging" => "Cargando",
                "Discharging" => "Descargando",
                "Full" => "Completa",
                "Not charging" => "Sin cargar",
                o => o,
            };
            Some(format!("{cap}% ({st})"))
        })
}

/// Versiones de los paquetes de `pkgver::WANTED` (dpkg y pacman desde disco,
/// rpm solo por nombre). Con el mismo fingerprint que el conteo de paquetes.
pub(crate) fn pkg_versions() -> HashMap<String, String> {
    let mut v = HashMap::new();
    if let Ok(text) = fs::read_to_string("/var/lib/dpkg/status") {
        v = pkgver::dpkg(&text, pkgver::WANTED);
    }
    if v.is_empty() {
        if let Ok(d) = fs::read_dir("/var/lib/pacman/local") {
            let names: Vec<String> = d
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            v = pkgver::pacman(names.iter().map(String::as_str), pkgver::WANTED);
        }
    }
    let has_rpm_db =
        Path::new("/var/lib/rpm").is_dir() || Path::new("/usr/lib/sysimage/rpm").is_dir();
    if v.is_empty() && has_rpm_db {
        let mut args = vec!["-q", "--qf", "%{NAME} %{VERSION}\n"];
        args.extend(pkgver::WANTED);
        if let Some(out) = run_with("rpm", &args, SLOW_DB, false) {
            v = pkgver::rpm(&out, pkgver::WANTED);
        }
    }
    v
}

fn packages_fp() -> String {
    let home = var("HOME").unwrap_or_default();
    let user_flatpak = format!("{home}/.local/share/flatpak/app");
    let nix_profile = format!("{home}/.nix-profile");
    let mut fp = cache::fingerprint(&[
        "/usr/lib/sysimage/rpm/rpmdb.sqlite",
        "/usr/lib/sysimage/rpm/rpmdb.sqlite-wal",
        "/var/lib/rpm/rpmdb.sqlite",
        "/var/lib/rpm/Packages",
        "/var/lib/dpkg/status",
        "/var/lib/pacman/local",
        "/lib/apk/db/installed",
        "/var/db/xbps",
        "/var/log/packages",
        "/var/lib/flatpak/app",
        "/var/lib/snapd/snaps",
        "/run/current-system/sw",
        "/nix/var/nix/profiles/default",
        nix_profile.as_str(),
        user_flatpak.as_str(),
    ]);
    fp.push_str(&cache::fingerprint_children("/var/db/pkg"));
    fp
}

fn gpu_fp() -> String {
    let mut ids: Vec<String> = fs::read_dir("/sys/class/drm")
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if !n.starts_with("card") || n.contains('-') {
                        return None;
                    }
                    let v = read(e.path().join("device/vendor")).unwrap_or_default();
                    let id = read(e.path().join("device/device")).unwrap_or_default();
                    let drv = fs::read_link(e.path().join("device/driver"))
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                        .unwrap_or_default();
                    Some(format!("{n}:{v}:{id}:{drv}"))
                })
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids.join(";")
}

pub fn gather(sys: &System, hidden: &[String], cache: &mut Cache) -> Info {
    let show = |m: &str| !is_hidden(hidden, m);
    let need_ver = show("de") || show("terminal");
    let pkg_fp = if show("packages") || need_ver {
        packages_fp()
    } else {
        String::new()
    };
    let versions = if need_ver {
        match cache.hit("pkgver", &pkg_fp) {
            Some(c) => pkgver::decode(&c),
            None => {
                let v = pkg_versions();
                cache.store("pkgver", &pkg_fp, &pkgver::encode(&v));
                v
            }
        }
    } else {
        HashMap::new()
    };
    let versions = &versions;
    let gpu_fp = if show("gpu") { gpu_fp() } else { String::new() };
    let pkg_hit = cache.hit("packages", &pkg_fp);
    let gpu_hit = cache.hit("gpu", &gpu_fp);
    let pkg_miss = pkg_hit.is_none();
    let gpu_miss = gpu_hit.is_none();
    let (pkg, de, gpu, theme, ip, wm) = thread::scope(|s| {
        let (a, b, c, d, e, f) = (
            s.spawn(|| {
                show("packages")
                    .then(|| pkg_hit.clone().or_else(packages))
                    .flatten()
            }),
            s.spawn(|| show("de").then(|| desktop(versions)).flatten()),
            s.spawn(|| show("gpu").then(|| gpu_hit.clone().or_else(gpus)).flatten()),
            s.spawn(|| {
                if show("theme") || show("icons") || show("font") {
                    appearance()
                } else {
                    Theme::default()
                }
            }),
            s.spawn(|| show("ip").then(local_ip).flatten()),
            s.spawn(|| show("wm").then(wm).flatten()),
        );
        (
            a.join().ok().flatten(),
            b.join().ok().flatten(),
            c.join().ok().flatten(),
            d.join().unwrap_or_default(),
            e.join().ok().flatten(),
            f.join().ok().flatten(),
        )
    });
    if pkg_miss {
        if let Some(v) = &pkg {
            cache.store("packages", &pkg_fp, v);
        }
    }
    if gpu_miss {
        if let Some(v) = &gpu {
            cache.store("gpu", &gpu_fp, v);
        }
    }
    let [theme, icons, fnt] = theme;
    let (id, like, pretty) = os_release();
    let os = pretty
        .or_else(System::long_os_version)
        .or_else(System::name)
        .unwrap_or_else(|| "Linux".into());
    let la = System::load_average();
    let sections = [
        section(
            "Sistema",
            hidden,
            vec![
                (
                    "os",
                    "OS",
                    show("os").then(|| format!("{os} {}", env::consts::ARCH)),
                ),
                (
                    "host",
                    "Host",
                    show("host")
                        .then(|| host_model().or_else(|| read("/proc/device-tree/model")))
                        .flatten(),
                ),
                (
                    "kernel",
                    "Kernel",
                    show("kernel").then(System::kernel_version).flatten(),
                ),
                (
                    "uptime",
                    "Uptime",
                    show("uptime").then(|| uptime(System::uptime())),
                ),
                (
                    "init",
                    "Init",
                    show("init").then(|| read("/proc/1/comm")).flatten(),
                ),
                (
                    "processes",
                    "Procesos",
                    show("processes").then(processes).flatten(),
                ),
                (
                    "load",
                    "Carga",
                    show("load").then(|| format!("{:.2} {:.2} {:.2}", la.one, la.five, la.fifteen)),
                ),
                ("packages", "Paquetes", pkg),
                ("shell", "Shell", show("shell").then(shell).flatten()),
                (
                    "terminal",
                    "Terminal",
                    show("terminal").then(|| terminal(versions)).flatten(),
                ),
                ("locale", "Idioma", show("locale").then(locale).flatten()),
                ("ip", "IP", ip),
            ],
        ),
        section(
            "Escritorio",
            hidden,
            vec![
                ("de", "DE", de),
                ("wm", "WM", wm),
                (
                    "display",
                    "Pantalla",
                    show("display").then(displays).flatten(),
                ),
                ("theme", "Tema", theme),
                ("icons", "Iconos", icons),
                ("font", "Fuente", fnt),
            ],
        ),
        section(
            "Hardware",
            hidden,
            vec![
                ("cpu", "CPU", show("cpu").then(|| cpu(sys)).flatten()),
                ("temp", "Temp", show("temp").then(cpu_temp).flatten()),
                ("gpu", "GPU", gpu),
                (
                    "battery",
                    "Batería",
                    show("battery").then(battery).flatten(),
                ),
            ],
        ),
    ];
    let user = var("USER")
        .or_else(|| var("LOGNAME"))
        .unwrap_or_else(|| "usuario".into());
    let host = System::host_name().unwrap_or_else(|| "localhost".into());
    Info {
        user: clean(&user),
        host: clean(&host),
        distro_id: id.unwrap_or_else(|| "linux".into()),
        id_like: like,
        sections: sections.into_iter().flatten().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_reads_output_larger_than_the_pipe_buffer() {
        let out = run("seq", &["1", "100000"]).expect("seq debería funcionar");
        assert_eq!(out.lines().count(), 100_000);
    }

    #[test]
    fn run_kills_commands_that_take_too_long() {
        let t = Instant::now();
        assert!(run("sleep", &["10"]).is_none());
        assert!(t.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn run_returns_none_for_failing_or_missing_commands() {
        assert!(run("false", &[]).is_none());
        assert!(run("rsi-comando-que-no-existe", &[]).is_none());
    }

    #[test]
    fn parse_ip_lists_global_addresses_and_skips_container_bridges() {
        let out = "2: enp1s0    inet 192.168.1.5/24 brd 192.168.1.255 scope global dynamic enp1s0\\       valid_lft 85000sec\n\
                   3: docker0    inet 172.17.0.1/16 brd 172.17.255.255 scope global docker0\\       valid_lft forever\n";
        assert_eq!(parse_ip(out).as_deref(), Some("enp1s0: 192.168.1.5/24"));
        assert_eq!(parse_ip(""), None);
    }

    #[test]
    fn hidden_modules_are_matched_exactly() {
        let hidden = vec!["ip".to_string()];
        assert!(is_hidden(&hidden, "ip"));
        assert!(!is_hidden(&hidden, "cpu"));
    }

    #[test]
    fn section_drops_hidden_rows_and_empty_sections() {
        let hidden = vec!["ip".to_string()];
        let rows = vec![("ip", "IP", Some("1.2.3.4".to_string()))];
        assert!(section("Red", &hidden, rows).is_none());
        let rows = vec![
            ("ip", "IP", Some("1.2.3.4".to_string())),
            ("os", "OS", Some("Linux".to_string())),
        ];
        let s = section("Sistema", &hidden, rows).unwrap();
        assert_eq!(s.rows, vec![("os", "OS", "Linux".to_string())]);
    }

    #[test]
    fn parse_version_finds_the_first_version_number() {
        let v = |l| parse_version(l).unwrap();
        assert_eq!(
            v("GNU bash, version 5.2.26(1)-release (x86_64-redhat-linux-gnu)"),
            "5.2.26"
        );
        assert_eq!(v("zsh 5.9 (x86_64-redhat-linux-gnu)"), "5.9");
        assert_eq!(v("fish, version 3.7.1"), "3.7.1");
        assert_eq!(parse_version("sin versión"), None);
    }

    #[test]
    fn find_wm_uses_the_priority_order_of_the_table() {
        let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<HashSet<_>>();
        assert_eq!(find_wm(&set(&["bash", "kwin_wayland", "i3"])), Some("KWin"));
        assert_eq!(find_wm(&set(&["Hyprland"])), Some("Hyprland"));
        assert_eq!(find_wm(&set(&["bash", "vim"])), None);
    }

    fn dtd_1080p60() -> Vec<u8> {
        let mut edid = vec![0u8; 128];
        edid[54..72].copy_from_slice(&[
            0x02, 0x3A, 0x80, 0x18, 0x71, 0x38, 0x2D, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        edid
    }

    #[test]
    fn edid_identity_reads_name_and_size() {
        let mut e = vec![0u8; 128];
        e[21] = 41;
        e[22] = 23;
        e[72..90].copy_from_slice(&[
            0, 0, 0, 0xFC, 0, b'B', b'P', b'C', b'-', b'2', b'0', b'W', b'R', 0x0A, 0x20, 0x20,
            0x20, 0x20,
        ]);
        assert_eq!(edid_identity(&e), (Some("BPC-20WR".to_string()), Some(19)));
        e[22] = 0;
        assert_eq!(edid_identity(&e).1, None);
        e[72 + 5] = 0x1B; // un escape nunca llega al texto
        assert_eq!(edid_identity(&e).0.as_deref(), Some("PC-20WR"));
        assert_eq!(edid_identity(&e[..64]), (None, None));
    }

    #[test]
    fn edid_refresh_reads_the_matching_timing_descriptor() {
        let edid = dtd_1080p60();
        assert_eq!(edid_refresh(&edid, "1920x1080"), Some(60));
        assert_eq!(edid_refresh(&edid, "1920x1080i"), Some(60));
        assert_eq!(edid_refresh(&edid, "1600x900"), None);
        assert_eq!(edid_refresh(&edid[..64], "1920x1080"), None);
        assert_eq!(edid_refresh(&edid, "raro"), None);
    }

    #[test]
    fn cpu_temp_picks_the_cpu_sensor_and_rounds() {
        let root = env::temp_dir().join(format!("rsi-hwmon-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let put = |dir: &str, file: &str, txt: &str| {
            fs::create_dir_all(root.join(dir)).unwrap();
            fs::write(root.join(dir).join(file), txt).unwrap();
        };
        put("hwmon0", "name", "nvme\n");
        put("hwmon0", "temp1_input", "30000\n");
        put("hwmon1", "name", "k10temp\n");
        put("hwmon1", "temp1_label", "Tctl\n");
        put("hwmon1", "temp1_input", "54500\n");
        assert_eq!(cpu_temp_in(&root).as_deref(), Some("55 °C"));
        assert_eq!(cpu_temp_in(&root.join("no-existe")), None);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn uptime_formats_days_hours_and_minutes() {
        assert_eq!(uptime(59), "0m");
        assert_eq!(uptime(3600), "1h 0m");
        assert_eq!(uptime(90_061), "1d 1h 1m");
    }

    #[test]
    fn font_extracts_name_and_size() {
        assert_eq!(font("Noto Sans,10,-1,5,50,0,0,0,0,0"), "Noto Sans (10pt)");
        assert_eq!(font("Inter"), "Inter");
        assert_eq!(font("Cantarell 11"), "Cantarell 11");
    }

    #[test]
    fn junk_detects_placeholder_dmi_values() {
        assert!(junk("To Be Filled By O.E.M."));
        assert!(junk("Default string"));
        assert!(junk("Unknown"));
        assert!(!junk("ThinkPad X1 Carbon"));
    }

    #[test]
    fn ini_reads_the_requested_section_and_key() {
        let path = env::temp_dir().join(format!("rsi-test-{}.ini", std::process::id()));
        fs::write(&path, "[A]\nk = uno\n[B]\nk = dos\nvacio =\n").unwrap();
        let p = path.to_string_lossy();
        assert_eq!(ini(&p, "A", "k").as_deref(), Some("uno"));
        assert_eq!(ini(&p, "B", "k").as_deref(), Some("dos"));
        assert_eq!(ini(&p, "B", "vacio"), None);
        assert_eq!(ini(&p, "C", "k"), None);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn fingerprints_are_stable_between_calls() {
        assert_eq!(packages_fp(), packages_fp());
        assert_eq!(gpu_fp(), gpu_fp());
    }
}
