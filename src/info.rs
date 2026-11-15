use crate::ui::clean;
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
    "display",
    "theme",
    "icons",
    "font",
    "cpu",
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
    pub rows: Vec<(&'static str, String)>,
}

pub struct Info {
    pub user: String,
    pub host: String,
    pub distro_id: String,
    pub id_like: Option<String>,
    pub sections: Vec<Section>,
}

type Theme = [Option<String>; 3];

fn var(k: &str) -> Option<String> {
    env::var(k).ok().filter(|v| !v.is_empty())
}

fn read(p: impl AsRef<Path>) -> Option<String> {
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

fn run(bin: &str, args: &[&str]) -> Option<String> {
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

    let limit = Duration::from_secs(2);
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
    status
        .success()
        .then(|| String::from_utf8_lossy(&buf).into_owned())
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

fn cfg(rel: &str) -> String {
    let base = var("XDG_CONFIG_HOME")
        .filter(|p| Path::new(p).is_absolute())
        .or_else(|| var("HOME").map(|h| format!("{h}/.config")));
    base.map_or_else(String::new, |b| format!("{b}/{rel}"))
}

type RawRow = (&'static str, &'static str, Option<String>);

fn section(title: &'static str, hidden: &[String], rows: Vec<RawRow>) -> Option<Section> {
    let rows: Vec<(&'static str, String)> = rows
        .into_iter()
        .filter(|(id, _, _)| !is_hidden(hidden, id))
        .filter_map(|(_, k, v)| v.map(|val| (k, clean(&val))))
        .filter(|(_, v)| !v.is_empty())
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
        lines(run(
            "nix-store",
            &["-q", "--requisites", "/run/current-system/sw"],
        ))
    } else {
        0
    };
    let counts = [
        (
            "rpm",
            if has_rpm_db {
                lines(run("rpm", &["-qa", "--qf", "x\n"]))
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

fn shell() -> Option<String> {
    let s = var("SHELL")?;
    Some(Path::new(&s).file_name()?.to_string_lossy().into_owned())
}

fn terminal() -> Option<String> {
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
    KNOWN
        .iter()
        .find(|(k, _)| var(k).is_some())
        .map(|(_, n)| n.to_string())
        .or_else(|| var("TERM"))
}

fn locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter().find_map(var)
}

fn desktop() -> Option<String> {
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

fn displays() -> Option<String> {
    let mut v: Vec<String> = fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let port = name.strip_prefix("card")?.split_once('-')?.1.to_string();
            let p = e.path();
            if read(p.join("status"))? != "connected" {
                return None;
            }
            let mode = read(p.join("modes"))?.lines().next()?.to_string();
            Some(format!("{port}: {mode}"))
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
        Some(p) if p != threads => format!("{p}C/{threads}T"),
        _ => format!("{threads}T"),
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

fn lspci_gpus() -> Option<String> {
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
    (!v.is_empty()).then(|| v.join(", "))
}

fn drm_gpus() -> Option<String> {
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
    (!v.is_empty()).then(|| v.join(", "))
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

fn gpus() -> Option<String> {
    lspci_gpus().or_else(drm_gpus)
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

pub fn gather(sys: &System, hidden: &[String]) -> Info {
    let show = |m: &str| !is_hidden(hidden, m);
    let (pkg, de, gpu, theme, ip) = thread::scope(|s| {
        let (a, b, c, d, e) = (
            s.spawn(|| show("packages").then(packages).flatten()),
            s.spawn(|| show("de").then(desktop).flatten()),
            s.spawn(|| show("gpu").then(gpus).flatten()),
            s.spawn(|| {
                if show("theme") || show("icons") || show("font") {
                    appearance()
                } else {
                    Theme::default()
                }
            }),
            s.spawn(|| show("ip").then(local_ip).flatten()),
        );
        (
            a.join().ok().flatten(),
            b.join().ok().flatten(),
            c.join().ok().flatten(),
            d.join().unwrap_or_default(),
            e.join().ok().flatten(),
        )
    });
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
                ("os", "OS", Some(format!("{os} {}", env::consts::ARCH))),
                (
                    "host",
                    "Host",
                    host_model().or_else(|| read("/proc/device-tree/model")),
                ),
                ("kernel", "Kernel", System::kernel_version()),
                ("uptime", "Uptime", Some(uptime(System::uptime()))),
                ("init", "Init", read("/proc/1/comm")),
                ("processes", "Procesos", processes()),
                (
                    "load",
                    "Carga",
                    Some(format!("{:.2} {:.2} {:.2}", la.one, la.five, la.fifteen)),
                ),
                ("packages", "Paquetes", pkg),
                ("shell", "Shell", shell()),
                ("terminal", "Terminal", terminal()),
                ("locale", "Idioma", locale()),
                ("ip", "IP", ip),
            ],
        ),
        section(
            "Escritorio",
            hidden,
            vec![
                ("de", "DE/WM", de),
                ("display", "Pantalla", displays()),
                ("theme", "Tema", theme),
                ("icons", "Iconos", icons),
                ("font", "Fuente", fnt),
            ],
        ),
        section(
            "Hardware",
            hidden,
            vec![
                ("cpu", "CPU", cpu(sys)),
                ("gpu", "GPU", gpu),
                ("battery", "Batería", battery()),
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
        assert_eq!(s.rows, vec![("OS", "Linux".to_string())]);
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
}
