//! Versiones de paquetes sin ejecutar nada extra: dpkg y pacman se leen de
//! disco y rpm se consulta solo por los nombres que interesan.
//!
//! Política de versión mostrada: parte "upstream", sin epoch (`1:`), sin
//! revisión de empaquetado (`-1pop1`) y sin sufijos de compilación
//! (`~1795032854~26.04~1de865b`, `+git2024`). Las marcas de preliminar
//! (`~beta9`, `~rc1`) se conservan.

use std::collections::HashMap;

/// Paquetes cuya versión puede mostrarse. Los nombres cambian entre distros:
/// el terminal de COSMIC es `cosmic-term` (Pop!_OS, Fedora) o
/// `cosmic-terminal` (Arch).
pub const WANTED: &[&str] = &[
    "cosmic-session",
    "cosmic-term",
    "cosmic-terminal",
    "cosmic-comp",
    "gnome-terminal",
    "gnome-console",
    "kgx",
    "ptyxis",
    "konsole",
    "kitty",
    "alacritty",
    "wezterm",
    "foot",
    "ghostty",
    "xfce4-terminal",
    "xfce4-session",
    "mate-terminal",
    "lxterminal",
    "lxqt-session",
    "tilix",
    "terminator",
    "qterminal",
    "rxvt-unicode",
    "xterm",
    "st",
    "sakura",
    "terminology",
    "deepin-terminal",
    "cinnamon",
    "mate-session-manager",
    "budgie-desktop",
    "sway",
    "hyprland",
    "niri",
    "i3-wm",
    "i3",
    "openbox",
    "bspwm",
    "awesome",
    "qtile",
];

/// `1:1.11.0-1` -> `1.11.0`; `1.11.0~1795032854~26.04~1de865b` -> `1.11.0`;
/// `0.1.0~beta9-1pop1` -> `0.1.0~beta9`.
pub fn clean_version(v: &str) -> String {
    let v = match v.split_once(':') {
        Some((epoch, rest)) if !epoch.is_empty() && epoch.bytes().all(|b| b.is_ascii_digit()) => {
            rest
        }
        _ => v,
    };
    let v = match v.rsplit_once('-') {
        Some((up, rev)) if rev.starts_with(|c: char| c.is_ascii_digit()) => up,
        _ => v,
    };
    // `~` seguido de dígito es una marca de compilación; seguido de letra, una preliminar.
    let mut cut = v.len();
    for (i, c) in v.char_indices() {
        let next = v[i + c.len_utf8()..].chars().next();
        if (c == '~' && next.is_some_and(|n| n.is_ascii_digit())) || c == '+' {
            cut = i;
            break;
        }
    }
    v[..cut].to_string()
}

/// Una pasada por `/var/lib/dpkg/status`. Solo cuentan las estrofas
/// `install ok installed`; con multiarch gana la primera del nombre.
pub fn dpkg(text: &str, wanted: &[&str]) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    let (mut pkg, mut ver, mut ok) = (None, None, false);
    for line in text.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if let (true, Some(p), Some(v)) = (ok, pkg, ver) {
                if wanted.contains(&p) {
                    versions
                        .entry(p.to_string())
                        .or_insert_with(|| clean_version(v));
                }
            }
            (pkg, ver, ok) = (None, None, false);
        } else if let Some(v) = line.strip_prefix("Package: ") {
            pkg = Some(v.trim());
        } else if let Some(v) = line.strip_prefix("Version: ") {
            ver = Some(v.trim());
        } else if let Some(v) = line.strip_prefix("Status: ") {
            ok = v.trim() == "install ok installed";
        }
    }
    versions
}

/// Nombres de directorio de `/var/lib/pacman/local` (`nombre-versión-release`).
/// El nombre puede llevar guiones; versión y release no, así que se parte por la derecha.
pub fn pacman<'a>(
    names: impl Iterator<Item = &'a str>,
    wanted: &[&str],
) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    for n in names {
        let mut it = n.rsplitn(3, '-');
        let (Some(_rel), Some(ver), Some(name)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        if wanted.contains(&name) {
            versions
                .entry(name.to_string())
                .or_insert_with(|| clean_version(ver));
        }
    }
    versions
}

/// Salida de `rpm -q --qf '%{NAME} %{VERSION}\n' paquetes…`. Los que no están
/// instalados salen como "package x is not installed" y se ignoran.
pub fn rpm(out: &str, wanted: &[&str]) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    for line in out.lines() {
        let mut it = line.split_whitespace();
        if let (Some(name), Some(ver), None) = (it.next(), it.next(), it.next()) {
            if wanted.contains(&name) {
                versions
                    .entry(name.to_string())
                    .or_insert_with(|| clean_version(ver));
            }
        }
    }
    versions
}

/// Formato de caché: `nombre=versión` separados por espacios.
pub fn encode(m: &HashMap<String, String>) -> String {
    let mut v: Vec<String> = m.iter().map(|(k, v)| format!("{k}={v}")).collect();
    v.sort();
    v.join(" ")
}

pub fn decode(s: &str) -> HashMap<String, String> {
    s.split_whitespace()
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = "\
Package: cosmic-term
Status: install ok installed
Version: 1.11.0~1794146128~26.04~b209f70
Architecture: amd64

Package: bash
Status: install ok installed
Version: 5.2.26-1

Package: kitty
Status: deinstall ok config-files
Version: 2.0-1
";

    #[test]
    fn dpkg_collects_only_installed_wanted_packages() {
        let v = dpkg(STATUS, WANTED);
        assert_eq!(v["cosmic-term"], "1.11.0");
        assert!(!v.contains_key("bash"));
        assert!(!v.contains_key("kitty"));
    }

    #[test]
    fn dpkg_multiarch_first_stanza_wins() {
        let s = "Package: kitty\nStatus: install ok installed\nVersion: 0.35-1\n\n\
                 Package: kitty\nStatus: install ok installed\nVersion: 0.34-1\n";
        assert_eq!(dpkg(s, WANTED)["kitty"], "0.35");
    }

    #[test]
    fn dpkg_last_stanza_without_trailing_blank_line() {
        let s = "Package: foot\nStatus: install ok installed\nVersion: 1.17-1";
        assert_eq!(dpkg(s, WANTED)["foot"], "1.17");
        assert!(dpkg("", WANTED).is_empty());
    }

    #[test]
    fn clean_version_applies_the_policy() {
        assert_eq!(clean_version("1:1.11.0-1"), "1.11.0");
        assert_eq!(clean_version("1.11.0~1795032854~26.04~1de865b"), "1.11.0");
        assert_eq!(clean_version("1:0.1.0~beta9-1pop1"), "0.1.0~beta9");
        assert_eq!(clean_version("5.2.26-1ubuntu1"), "5.2.26");
        assert_eq!(clean_version("1.2+git20240101-1"), "1.2");
        assert_eq!(clean_version("2.3"), "2.3");
        assert_eq!(clean_version("1.0.0-beta"), "1.0.0-beta");
        assert_eq!(clean_version("abc:1.0"), "abc:1.0");
        assert_eq!(clean_version(""), "");
    }

    #[test]
    fn pacman_splits_from_the_right() {
        let names = [
            "cosmic-terminal-1:1.11.0-1",
            "lib32-gcc-libs-13.2.1-3",
            "kitty-0.35.2-1",
            "ALPM_DB_VERSION",
        ];
        let v = pacman(names.iter().copied(), WANTED);
        assert_eq!(v["cosmic-terminal"], "1.11.0");
        assert_eq!(v["kitty"], "0.35.2");
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn rpm_ignores_missing_packages() {
        let out = "cosmic-term 1.0.0\npackage kitty is not installed\n";
        let v = rpm(out, WANTED);
        assert_eq!(v["cosmic-term"], "1.0.0");
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn cache_encoding_round_trips() {
        let mut m = HashMap::new();
        m.insert("kitty".to_string(), "0.35.2".to_string());
        m.insert("cosmic-term".to_string(), "1.11.0~beta1".to_string());
        assert_eq!(decode(&encode(&m)), m);
        assert!(decode("").is_empty());
    }
}
