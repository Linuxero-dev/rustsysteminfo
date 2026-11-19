use crate::{completions, info, logo, ui};
use std::fs;

pub const HELP: &str = "rsi — información del sistema y monitor en vivo

USO: rsi [OPCIONES]

General:
  -h, --help            muestra esta ayuda
  -V, --version         muestra la versión
  --once                imprime una vez y sale (sin monitor en vivo)
  --no-anim             logo estático
  --json                imprime los datos en JSON y sale
  --no-cache            recalcula todo ignorando la caché

Logo:
  --logo NOMBRE         fuerza un logo (ver --list-logos)
  --no-logo             no muestra el logo
  --list-logos          lista los logos disponibles

Módulos:
  --no-MÓDULO           oculta un módulo, p. ej. --no-ip o --no-gpu
  --hide A,B            oculta varios módulos a la vez
  --show A,B            muestra módulos que la configuración oculta
  --list-modules        lista los módulos disponibles

Configuración (~/.config/rsi/config):
  --config RUTA         usa otro fichero de configuración
  --no-config           ignora el fichero de configuración
  --gen-config          imprime una configuración de ejemplo

Diagnóstico:
  --report              imprime un informe para adjuntar a un issue

Shell:
  --completions SH      imprime el completado para bash, zsh o fish

Por defecto permanece abierto (Enter, Esc o q para salir).
Respeta la variable NO_COLOR.
Caché en ~/.cache/rsi: se invalida al cambiar el sistema o al reiniciar.";

pub const SAMPLE_CONFIG: &str = "# Configuración de rsi (~/.config/rsi/config)
# Las opciones de la línea de comandos tienen prioridad sobre este fichero.
# Quita el # inicial de las líneas que quieras activar.

# Módulos que se ocultan, separados por comas (rsi --list-modules los muestra).
# hide = ip,gpu

# Logo fijo (rsi --list-logos). Por defecto se detecta la distribución.
# logo = arch

# Oculta el logo.
# no_logo = false

# Animación del logo.
# animate = true
";

const MAX_CONFIG_BYTES: u64 = 64 * 1024;

#[derive(Debug, PartialEq)]
pub struct Cli {
    pub once: bool,
    pub animate: bool,
    pub no_logo: bool,
    pub json: bool,
    pub no_cache: bool,
    pub logo: Option<String>,
    pub hidden: Vec<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            once: false,
            animate: true,
            no_logo: false,
            json: false,
            no_cache: false,
            logo: None,
            hidden: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Run(Cli),
    Help,
    Version,
    ListLogos,
    ListModules,
    GenConfig,
    Report,
    Completions(String),
}

#[derive(Debug, PartialEq)]
pub enum ConfigSource {
    Off,
    Default,
    Path(String),
}

pub fn hide(hidden: &mut Vec<String>, module: &str) -> Result<(), String> {
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

fn show(hidden: &mut Vec<String>, module: &str) -> Result<(), String> {
    let m = module.to_ascii_lowercase();
    if !info::MODULES.contains(&m.as_str()) {
        return Err(format!(
            "Módulo desconocido: {} (ver --list-modules)",
            ui::clean(module)
        ));
    }
    hidden.retain(|h| *h != m);
    Ok(())
}

/// Qué fichero de configuración usar según `--config` y `--no-config` (gana el último).
pub fn config_source(args: &[String]) -> ConfigSource {
    let mut src = ConfigSource::Default;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--no-config" {
            src = ConfigSource::Off;
        } else if a == "--config" {
            if let Some(p) = it.next() {
                src = ConfigSource::Path(p.clone());
            }
        } else if let Some(p) = a.strip_prefix("--config=") {
            src = ConfigSource::Path(p.to_string());
        }
    }
    src
}

fn set_bool(slot: &mut bool, value: &str, line: usize, warns: &mut Vec<String>) {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => *slot = true,
        "false" | "no" | "off" | "0" => *slot = false,
        _ => warns.push(format!(
            "línea {line}: se esperaba true o false, no {}",
            ui::clean(value)
        )),
    }
}

/// Aplica el texto de un fichero de configuración sobre `cli` y devuelve los avisos.
pub fn apply_config(cli: &mut Cli, text: &str) -> Vec<String> {
    let mut warns = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            warns.push(format!("línea {n}: se esperaba «clave = valor»"));
            continue;
        };
        let key = k.trim().to_ascii_lowercase();
        let v = v.trim().trim_matches('"').trim_matches('\'');
        match key.as_str() {
            "hide" => {
                for m in v.split(',').map(str::trim).filter(|m| !m.is_empty()) {
                    if let Err(e) = hide(&mut cli.hidden, m) {
                        warns.push(format!("línea {n}: {e}"));
                    }
                }
            }
            "logo" => {
                if logo::by_name(v).is_some() {
                    cli.logo = Some(v.to_string());
                } else {
                    warns.push(format!("línea {n}: logo desconocido: {}", ui::clean(v)));
                }
            }
            "no_logo" => set_bool(&mut cli.no_logo, v, n, &mut warns),
            "animate" => set_bool(&mut cli.animate, v, n, &mut warns),
            _ => warns.push(format!("línea {n}: clave desconocida: {}", ui::clean(&key))),
        }
    }
    warns
}

fn read_limited(path: &str) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("no es un fichero regular".into());
    }
    if meta.len() > MAX_CONFIG_BYTES {
        return Err("demasiado grande (máximo 64 KiB)".into());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Carga la configuración. Un fichero por defecto que no existe no es un error;
/// uno indicado con `--config` que no se puede leer, sí.
pub fn load_config(src: &ConfigSource, default_path: &str) -> Result<(Cli, Vec<String>), String> {
    let mut cli = Cli::default();
    let (path, explicit) = match src {
        ConfigSource::Off => return Ok((cli, Vec::new())),
        ConfigSource::Default => (default_path, false),
        ConfigSource::Path(p) => (p.as_str(), true),
    };
    if path.is_empty() {
        return Ok((cli, Vec::new()));
    }
    match read_limited(path) {
        Ok(text) => {
            let warns = apply_config(&mut cli, &text);
            Ok((cli, warns))
        }
        Err(e) if explicit => Err(format!(
            "No se pudo leer la configuración {}: {e}",
            ui::clean(path)
        )),
        Err(_) => Ok((cli, Vec::new())),
    }
}

pub fn parse_args(args: Vec<String>, base: Cli) -> Result<Action, String> {
    let mut cli = base;
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n, Some(v.to_string())),
            _ => (arg.as_str(), None),
        };
        let takes_value = matches!(
            name,
            "--logo" | "--hide" | "--show" | "--config" | "--completions"
        );
        if inline.is_some() && !takes_value {
            return Err(format!("Opción desconocida: {}", ui::clean(&arg)));
        }
        match name {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--list-logos" => return Ok(Action::ListLogos),
            "--list-modules" => return Ok(Action::ListModules),
            "--gen-config" => return Ok(Action::GenConfig),
            "--report" => return Ok(Action::Report),
            "--once" | "--no-live" => cli.once = true,
            "--json" => cli.json = true,
            "--no-anim" | "--static" => cli.animate = false,
            "--no-logo" => cli.no_logo = true,
            "--no-cache" => cli.no_cache = true,
            "--no-config" => {}
            "--config" => {
                inline
                    .or_else(|| it.next())
                    .ok_or("--config necesita una ruta")?;
            }
            "--completions" => {
                let v = inline
                    .or_else(|| it.next())
                    .ok_or("--completions necesita un shell (bash, zsh o fish)")?;
                if !completions::SHELLS.contains(&v.as_str()) {
                    return Err(format!(
                        "Shell no soportado: {} (bash, zsh o fish)",
                        ui::clean(&v)
                    ));
                }
                return Ok(Action::Completions(v));
            }
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
            "--hide" | "--show" => {
                let v = inline
                    .or_else(|| it.next())
                    .ok_or("--hide y --show necesitan una lista de módulos (ver --list-modules)")?;
                for m in v.split(',').filter(|m| !m.is_empty()) {
                    if name == "--hide" {
                        hide(&mut cli.hidden, m)?;
                    } else {
                        show(&mut cli.hidden, m)?;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn run_cli(a: &[&str]) -> Cli {
        match parse_args(args(a), Cli::default()) {
            Ok(Action::Run(c)) => c,
            other => panic!("se esperaba Run, salió {other:?}"),
        }
    }

    fn parse(a: &[&str]) -> Result<Action, String> {
        parse_args(args(a), Cli::default())
    }

    #[test]
    fn defaults_run_the_live_monitor() {
        let c = run_cli(&[]);
        assert!(c.animate && !c.once && !c.no_logo && !c.json && !c.no_cache);
        assert!(c.logo.is_none() && c.hidden.is_empty());
    }

    #[test]
    fn simple_actions() {
        assert_eq!(parse(&["-h"]), Ok(Action::Help));
        assert_eq!(parse(&["--version"]), Ok(Action::Version));
        assert_eq!(parse(&["--list-logos"]), Ok(Action::ListLogos));
        assert_eq!(parse(&["--list-modules"]), Ok(Action::ListModules));
        assert_eq!(parse(&["--gen-config"]), Ok(Action::GenConfig));
        assert_eq!(parse(&["--report"]), Ok(Action::Report));
        assert_eq!(
            parse(&["--completions", "fish"]),
            Ok(Action::Completions("fish".into()))
        );
        assert!(parse(&["--completions", "tcsh"]).is_err());
        assert!(parse(&["--completions"]).is_err());
    }

    #[test]
    fn hides_modules_with_no_prefix_and_hide_list() {
        assert_eq!(run_cli(&["--no-ip"]).hidden, vec!["ip"]);
        assert_eq!(
            run_cli(&["--no-gpu", "--hide", "IP,disks", "--hide=ip"]).hidden,
            vec!["gpu", "ip", "disks"]
        );
    }

    #[test]
    fn show_undoes_a_hidden_module() {
        let base = Cli {
            hidden: vec!["ip".into(), "gpu".into()],
            ..Cli::default()
        };
        match parse_args(args(&["--show", "ip"]), base) {
            Ok(Action::Run(c)) => assert_eq!(c.hidden, vec!["gpu"]),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn keeps_the_flags_that_start_with_no() {
        let c = run_cli(&[
            "--no-anim",
            "--no-logo",
            "--no-live",
            "--no-config",
            "--no-cache",
        ]);
        assert!(!c.animate && c.no_logo && c.once && c.no_cache);
        assert!(c.hidden.is_empty());
        assert!(run_cli(&["--json"]).json);
    }

    #[test]
    fn logo_accepts_known_names_only() {
        assert_eq!(run_cli(&["--logo", "arch"]).logo.as_deref(), Some("arch"));
        assert_eq!(run_cli(&["--logo=tux"]).logo.as_deref(), Some("tux"));
        assert!(parse(&["--logo", "no-existe"]).is_err());
        assert!(parse(&["--logo"]).is_err());
    }

    #[test]
    fn rejects_unknown_options_and_modules() {
        assert!(parse(&["--nope"]).is_err());
        assert!(parse(&["--no-nada"]).is_err());
        assert!(parse(&["--hide", "ip,nada"]).is_err());
        assert!(parse(&["--once=1"]).is_err());
        assert!(parse(&["--no-cache=1"]).is_err());
        assert!(parse(&["--hide"]).is_err());
        assert!(parse(&["--config"]).is_err());
    }

    #[test]
    fn config_source_prefers_the_last_option() {
        assert_eq!(config_source(&args(&[])), ConfigSource::Default);
        assert_eq!(config_source(&args(&["--no-config"])), ConfigSource::Off);
        assert_eq!(
            config_source(&args(&["--no-config", "--config", "a.conf"])),
            ConfigSource::Path("a.conf".into())
        );
        assert_eq!(
            config_source(&args(&["--config=b.conf", "--no-config"])),
            ConfigSource::Off
        );
    }

    #[test]
    fn config_applies_known_keys_and_reports_the_rest() {
        let mut cli = Cli::default();
        let warns = apply_config(
            &mut cli,
            "# comentario\nhide = IP, gpu\nlogo = \"arch\"\nno_logo = yes\nanimate = off\nbasura\nfoo = 1\nhide = nada\nlogo = zzz\nanimate = quizá\n",
        );
        assert_eq!(cli.hidden, vec!["ip", "gpu"]);
        assert_eq!(cli.logo.as_deref(), Some("arch"));
        assert!(cli.no_logo && !cli.animate);
        assert_eq!(warns.len(), 5, "{warns:?}");
        assert!(warns[0].starts_with("línea 6"));
    }

    #[test]
    fn command_line_overrides_the_config() {
        let mut base = Cli::default();
        apply_config(&mut base, "logo = arch\nhide = ip\n");
        match parse_args(args(&["--logo", "fedora", "--show", "ip"]), base) {
            Ok(Action::Run(c)) => {
                assert_eq!(c.logo.as_deref(), Some("fedora"));
                assert!(c.hidden.is_empty());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn load_config_handles_missing_default_and_missing_explicit_files() {
        let dir = env::temp_dir().join(format!("rsi-cfg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config");
        let path = file.to_string_lossy().into_owned();
        let (c, w) = load_config(&ConfigSource::Default, &path).unwrap();
        assert_eq!(c, Cli::default());
        assert!(w.is_empty());
        assert!(load_config(&ConfigSource::Path(path.clone()), "").is_err());
        fs::write(&file, "hide = ip\n").unwrap();
        let (c, _) = load_config(&ConfigSource::Path(path.clone()), "").unwrap();
        assert_eq!(c.hidden, vec!["ip"]);
        let (c, _) = load_config(&ConfigSource::Off, &path).unwrap();
        assert!(c.hidden.is_empty());
        fs::write(&file, vec![b'#'; 70 * 1024]).unwrap();
        assert!(load_config(&ConfigSource::Path(path), "").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_sample_config_is_valid() {
        let mut cli = Cli::default();
        let uncommented: String = SAMPLE_CONFIG
            .lines()
            .map(|l| {
                l.strip_prefix("# ")
                    .filter(|r| r.contains(" = "))
                    .unwrap_or(l)
            })
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(apply_config(&mut cli, &uncommented).is_empty());
        assert_eq!(cli.hidden, vec!["ip", "gpu"]);
    }
}
