//! Terminal real a partir de los procesos padre (`/proc`), para los emuladores
//! que no dejan variables de entorno reconocibles (COSMIC Terminal, Ptyxis, foot…).

use std::fs;
use std::path::Path;

const MAX_DEPTH: usize = 32;

/// (nombre del ejecutable, nombre mostrado, paquetes que dan su versión).
/// El nombre se compara completo: `st` no coincide con `stty`, y `footclient`
/// no es `foot`.
pub const TERMINALS: &[(&str, &str, &[&str])] = &[
    (
        "cosmic-term",
        "cosmic-term",
        &["cosmic-term", "cosmic-terminal"],
    ),
    (
        "gnome-terminal-server",
        "GNOME Terminal",
        &["gnome-terminal"],
    ),
    ("gnome-terminal", "GNOME Terminal", &["gnome-terminal"]),
    ("ptyxis", "Ptyxis", &["ptyxis"]),
    ("ptyxis-agent", "Ptyxis", &["ptyxis"]),
    ("kgx", "GNOME Console", &["gnome-console", "kgx"]),
    ("konsole", "Konsole", &["konsole"]),
    ("yakuake", "Yakuake", &[]),
    ("kitty", "kitty", &["kitty"]),
    ("alacritty", "Alacritty", &["alacritty"]),
    ("wezterm-gui", "WezTerm", &["wezterm"]),
    ("wezterm", "WezTerm", &["wezterm"]),
    ("foot", "foot", &["foot"]),
    ("ghostty", "Ghostty", &["ghostty"]),
    ("xfce4-terminal", "Xfce Terminal", &["xfce4-terminal"]),
    ("mate-terminal", "MATE Terminal", &["mate-terminal"]),
    ("lxterminal", "LXTerminal", &["lxterminal"]),
    ("qterminal", "QTerminal", &["qterminal"]),
    ("tilix", "Tilix", &["tilix"]),
    ("terminator", "Terminator", &["terminator"]),
    ("sakura", "Sakura", &["sakura"]),
    ("terminology", "Terminology", &["terminology"]),
    ("deepin-terminal", "Deepin Terminal", &["deepin-terminal"]),
    ("urxvt", "rxvt-unicode", &["rxvt-unicode"]),
    ("urxvtd", "rxvt-unicode", &["rxvt-unicode"]),
    ("xterm", "XTerm", &["xterm"]),
    ("st", "st", &["st"]),
    ("tilda", "Tilda", &[]),
    ("guake", "Guake", &[]),
    ("rio", "Rio", &[]),
    ("contour", "Contour", &[]),
    ("zutty", "Zutty", &[]),
    ("cool-retro-term", "cool-retro-term", &[]),
    ("hyper", "Hyper", &[]),
];

/// Procesos que cortan la búsqueda: más arriba ya no está el terminal del
/// usuario (sesión remota o un servidor desprendido).
const STOPPERS: &[&str] = &[
    "sshd",
    "sshd-session",
    "login",
    "mosh-server",
    "systemd",
    "init",
];

/// PID del padre a partir del contenido de `/proc/<pid>/stat`. El nombre va
/// entre paréntesis y puede contener espacios y paréntesis: se busca el último `)`.
pub fn parse_ppid(stat: &str) -> Option<u32> {
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().nth(1)?.parse().ok()
}

fn base(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}

/// Nombres con los que puede conocerse un proceso: ejecutable, `comm` (truncado
/// a 15 caracteres) y los dos primeros elementos de `cmdline` (intérpretes como
/// `python3 /usr/bin/terminator`).
fn names(root: &Path, pid: u32) -> Vec<String> {
    let dir = root.join(pid.to_string());
    let mut v = Vec::new();
    if let Ok(exe) = fs::read_link(dir.join("exe")) {
        v.push(
            base(&exe.to_string_lossy())
                .trim_end_matches(" (deleted)")
                .to_string(),
        );
    }
    if let Ok(c) = fs::read_to_string(dir.join("comm")) {
        v.push(c.trim().to_string());
    }
    if let Ok(raw) = fs::read(dir.join("cmdline")) {
        for part in raw.split(|b| *b == 0).take(2) {
            let s = String::from_utf8_lossy(part);
            if !s.is_empty() {
                v.push(base(&s).to_string());
            }
        }
    }
    v
}

/// Entrada de `TERMINALS` que corresponde a alguno de `names`.
pub fn match_terminal(
    names: &[String],
) -> Option<&'static (&'static str, &'static str, &'static [&'static str])> {
    names
        .iter()
        .find_map(|n| TERMINALS.iter().find(|t| t.0 == n.as_str()))
}

fn is_stopper(names: &[String]) -> bool {
    names.iter().any(|n| STOPPERS.contains(&n.as_str()))
}

/// Cadena de nombres de ejecutable de los ancestros de `start` (para `--report`).
pub fn chain(root: &Path, start: u32) -> Vec<String> {
    let mut out = Vec::new();
    let mut pid = start;
    for _ in 0..MAX_DEPTH {
        if pid <= 1 {
            break;
        }
        let n = names(root, pid);
        out.push(n.first().cloned().unwrap_or_else(|| "?".into()));
        let Some(ppid) = fs::read_to_string(root.join(pid.to_string()).join("stat"))
            .ok()
            .and_then(|s| parse_ppid(&s))
        else {
            break;
        };
        pid = ppid;
    }
    out
}

/// Sube desde `start` hasta encontrar un terminal conocido. `None` en sesiones
/// remotas, o si la cadena se corta antes (p. ej. dentro de tmux).
pub fn find_terminal(
    root: &Path,
    start: u32,
) -> Option<&'static (&'static str, &'static str, &'static [&'static str])> {
    let mut pid = start;
    for _ in 0..MAX_DEPTH {
        if pid <= 1 {
            return None;
        }
        let n = names(root, pid);
        if let Some(t) = match_terminal(&n) {
            return Some(t);
        }
        if is_stopper(&n) {
            return None;
        }
        pid = fs::read_to_string(root.join(pid.to_string()).join("stat"))
            .ok()
            .and_then(|s| parse_ppid(&s))?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn ppid_survives_odd_process_names() {
        assert_eq!(parse_ppid("123 (bash) S 45 123 123 0 -1"), Some(45));
        assert_eq!(parse_ppid("123 (a b) c) S 77 1 1"), Some(77));
        assert_eq!(parse_ppid("garbage"), None);
        assert_eq!(parse_ppid("1 (x) S"), None);
    }

    #[test]
    fn full_names_only() {
        let n = |s: &str| vec![s.to_string()];
        assert_eq!(match_terminal(&n("st")).map(|t| t.1), Some("st"));
        assert!(match_terminal(&n("stty")).is_none());
        assert!(match_terminal(&n("footclient")).is_none());
        assert!(match_terminal(&n("gnome-terminal-")).is_none());
        assert_eq!(
            match_terminal(&n("gnome-terminal-server")).map(|t| t.1),
            Some("GNOME Terminal")
        );
    }

    fn fake(root: &Path, pid: u32, ppid: u32, exe: &str, comm: &str) {
        let d = root.join(pid.to_string());
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("stat"), format!("{pid} ({comm}) S {ppid} 1 1\n")).unwrap();
        fs::write(d.join("comm"), format!("{comm}\n")).unwrap();
        symlink(format!("/usr/bin/{exe}"), d.join("exe")).unwrap();
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("rsi-proctree-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn finds_the_terminal_even_with_a_truncated_comm() {
        let r = tmp("trunc");
        fake(&r, 500, 400, "rsi", "rsi");
        fake(&r, 400, 300, "bash", "bash");
        fake(&r, 300, 200, "gnome-terminal-server", "gnome-terminal-");
        fake(&r, 200, 1, "systemd", "systemd");
        let t = find_terminal(&r, 500).unwrap();
        assert_eq!(t.1, "GNOME Terminal");
        assert_eq!(
            chain(&r, 500),
            ["rsi", "bash", "gnome-terminal-server", "systemd"]
        );
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn a_remote_session_has_no_local_terminal() {
        let r = tmp("ssh");
        fake(&r, 500, 400, "rsi", "rsi");
        fake(&r, 400, 300, "bash", "bash");
        fake(&r, 300, 200, "sshd", "sshd");
        fake(&r, 200, 150, "kitty", "kitty");
        assert!(find_terminal(&r, 500).is_none());
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn loops_and_missing_parents_end_the_search() {
        let r = tmp("loop");
        fake(&r, 10, 20, "bash", "bash");
        fake(&r, 20, 10, "bash", "bash");
        assert!(find_terminal(&r, 10).is_none());
        assert!(find_terminal(&r, 999).is_none());
        let _ = fs::remove_dir_all(&r);
    }
}
