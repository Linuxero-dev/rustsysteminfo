//! `rsi --report`: texto para pegar en un issue cuando algo no se detecta.
//! No incluye usuario, nombre de equipo, IP ni números de serie.

use crate::info::{self, Info};
use crate::{pkgver, proctree};
use std::fmt::Write;
use std::fs;
use std::path::Path;

const ENV: &[&str] = &[
    "TERM",
    "TERM_PROGRAM",
    "COLORTERM",
    "XDG_CURRENT_DESKTOP",
    "XDG_SESSION_TYPE",
    "XDG_SESSION_DESKTOP",
    "DESKTOP_SESSION",
    "WAYLAND_DISPLAY",
    "DISPLAY",
    "LANG",
];

fn sysfs_names(dir: &str, file: &str) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    let val = info::read(e.path().join(file))?;
                    Some(format!("{n}: {val}"))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn gpus() -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir("/sys/class/drm")
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if !n.starts_with("card") || n.contains('-') {
                        return None;
                    }
                    let dev = e.path().join("device");
                    let driver = fs::read_link(dev.join("driver"))
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                        .unwrap_or_else(|| "?".into());
                    Some(format!(
                        "{n}: vendor={} device={} driver={driver}",
                        info::read(dev.join("vendor")).unwrap_or_else(|| "?".into()),
                        info::read(dev.join("device")).unwrap_or_else(|| "?".into()),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn section(o: &mut String, title: &str) {
    let _ = writeln!(o, "\n## {title}");
}

pub fn render(info: &Info) -> String {
    let mut o = String::new();
    let _ = writeln!(o, "# rsi {} --report", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(o, "arch: {}", std::env::consts::ARCH);

    section(&mut o, "Detectado");
    for s in &info.sections {
        for (id, label, value) in &s.rows {
            let _ = writeln!(o, "{id} ({label}): {value}");
        }
    }

    section(&mut o, "os-release");
    for k in ["ID", "ID_LIKE", "VERSION_ID", "PRETTY_NAME", "VARIANT_ID"] {
        if let Some(line) = info::read("/etc/os-release").and_then(|t| {
            t.lines()
                .find(|l| l.starts_with(&format!("{k}=")))
                .map(String::from)
        }) {
            let _ = writeln!(o, "{line}");
        }
    }

    section(&mut o, "Entorno");
    for k in ENV {
        let _ = writeln!(o, "{k}={}", info::var(k).unwrap_or_default());
    }

    section(&mut o, "Procesos padre");
    let _ = writeln!(
        o,
        "{}",
        proctree::chain(Path::new("/proc"), std::process::id()).join(" < ")
    );

    section(&mut o, "Versiones de paquetes");
    let mut v: Vec<_> = info::pkg_versions().into_iter().collect();
    v.sort();
    for (k, ver) in v {
        let _ = writeln!(o, "{k} {ver}");
    }
    let _ = writeln!(o, "(lista buscada: {} paquetes)", pkgver::WANTED.len());

    section(&mut o, "GPU (sysfs)");
    for l in gpus() {
        let _ = writeln!(o, "{l}");
    }
    if let Some(out) = info::run("lspci", &["-mm"]) {
        for l in out
            .lines()
            .filter(|l| ["VGA", "3D", "Display"].iter().any(|k| l.contains(k)))
        {
            let _ = writeln!(o, "lspci: {l}");
        }
    }

    section(&mut o, "hwmon");
    for l in sysfs_names("/sys/class/hwmon", "name") {
        let _ = writeln!(o, "{l}");
    }
    section(&mut o, "power_supply");
    for l in sysfs_names("/sys/class/power_supply", "type") {
        let _ = writeln!(o, "{l}");
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_has_no_personal_sections() {
        let info = Info {
            user: "alguien".into(),
            host: "mi-equipo".into(),
            distro_id: "linux".into(),
            id_like: None,
            sections: Vec::new(),
        };
        let r = render(&info);
        assert!(r.starts_with("# rsi "));
        assert!(!r.contains("alguien") && !r.contains("mi-equipo"));
        assert!(r.contains("## Procesos padre") && r.contains("## GPU (sysfs)"));
    }
}
