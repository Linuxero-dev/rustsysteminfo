use crate::info::MODULES;
use crate::logo::NAMES;

pub const SHELLS: &[&str] = &["bash", "zsh", "fish"];

const FLAGS: &[(&str, &str)] = &[
    ("help", "muestra la ayuda"),
    ("version", "muestra la version"),
    ("once", "imprime una vez y sale"),
    ("no-anim", "logo estatico"),
    ("no-logo", "no muestra el logo"),
    ("json", "imprime los datos en JSON"),
    ("no-cache", "recalcula todo ignorando la cache"),
    ("list-logos", "lista los logos"),
    ("list-modules", "lista los modulos"),
    ("gen-config", "imprime una configuracion de ejemplo"),
    ("report", "imprime un informe para adjuntar a un issue"),
    ("no-config", "ignora el fichero de configuracion"),
];

const VALUE_FLAGS: &[(&str, &str)] = &[
    ("logo", "fuerza un logo"),
    ("hide", "oculta modulos"),
    ("show", "muestra modulos ocultos por la configuracion"),
    ("config", "usa otro fichero de configuracion"),
    ("completions", "imprime el completado para un shell"),
];

fn hide_flags() -> Vec<String> {
    MODULES.iter().map(|m| format!("no-{m}")).collect()
}

fn bash() -> String {
    let mut words: Vec<String> = vec!["-h".into(), "-V".into()];
    words.extend(FLAGS.iter().map(|(f, _)| format!("--{f}")));
    words.extend(VALUE_FLAGS.iter().map(|(f, _)| format!("--{f}")));
    words.extend(hide_flags().iter().map(|f| format!("--{f}")));
    "# Completado de rsi para bash.
_rsi() {
    local cur prev
    cur=\"${COMP_WORDS[COMP_CWORD]}\"
    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"
    case \"$prev\" in
        --logo) mapfile -t COMPREPLY < <(compgen -W \"@LOGOS@\" -- \"$cur\"); return ;;
        --hide|--show) mapfile -t COMPREPLY < <(compgen -W \"@MODULES@\" -- \"$cur\"); return ;;
        --completions) mapfile -t COMPREPLY < <(compgen -W \"bash zsh fish\" -- \"$cur\"); return ;;
        --config) mapfile -t COMPREPLY < <(compgen -f -- \"$cur\"); return ;;
    esac
    mapfile -t COMPREPLY < <(compgen -W \"@FLAGS@\" -- \"$cur\")
}
complete -F _rsi rsi
"
    .replace("@LOGOS@", &NAMES.join(" "))
    .replace("@MODULES@", &MODULES.join(" "))
    .replace("@FLAGS@", &words.join(" "))
}

fn zsh() -> String {
    let mut specs: Vec<String> = vec![
        "'-h[muestra la ayuda]'".into(),
        "'-V[muestra la version]'".into(),
    ];
    specs.extend(FLAGS.iter().map(|(f, d)| format!("'--{f}[{d}]'")));
    specs.push(format!(
        "'--logo[fuerza un logo]:logo:({})'",
        NAMES.join(" ")
    ));
    for (f, d) in &VALUE_FLAGS[1..3] {
        specs.push(format!("'--{f}[{d}]:modulos:({})'", MODULES.join(" ")));
    }
    specs.push("'--config[usa otro fichero de configuracion]:fichero:_files'".into());
    specs.push("'--completions[imprime el completado para un shell]:shell:(bash zsh fish)'".into());
    specs.extend(
        MODULES
            .iter()
            .map(|m| format!("'--no-{m}[oculta el modulo {m}]'")),
    );
    format!(
        "#compdef rsi\n# Completado de rsi para zsh.\n_arguments -s \\\n  {}\n",
        specs.join(" \\\n  ")
    )
}

fn fish() -> String {
    let mut o = String::from("# Completado de rsi para fish.\n");
    o.push_str("complete -c rsi -s h -l help -d 'muestra la ayuda'\n");
    o.push_str("complete -c rsi -s V -l version -d 'muestra la version'\n");
    for (f, d) in &FLAGS[2..] {
        o.push_str(&format!("complete -c rsi -l {f} -d '{d}'\n"));
    }
    o.push_str(&format!(
        "complete -c rsi -l logo -x -a '{}' -d 'fuerza un logo'\n",
        NAMES.join(" ")
    ));
    for (f, d) in &VALUE_FLAGS[1..3] {
        o.push_str(&format!(
            "complete -c rsi -l {f} -x -a '{}' -d '{d}'\n",
            MODULES.join(" ")
        ));
    }
    o.push_str("complete -c rsi -l config -r -d 'usa otro fichero de configuracion'\n");
    o.push_str("complete -c rsi -l completions -x -a 'bash zsh fish' -d 'imprime el completado'\n");
    for m in MODULES {
        o.push_str(&format!(
            "complete -c rsi -l no-{m} -d 'oculta el modulo {m}'\n"
        ));
    }
    o
}

pub fn script(shell: &str) -> Option<String> {
    match shell {
        "bash" => Some(bash()),
        "zsh" => Some(zsh()),
        "fish" => Some(fish()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{parse_args, Cli};

    #[test]
    fn every_listed_flag_is_accepted_by_the_parser() {
        for (f, _) in FLAGS {
            assert!(
                parse_args(vec![format!("--{f}")], Cli::default()).is_ok(),
                "--{f}"
            );
        }
        for (f, _) in VALUE_FLAGS {
            let value = match *f {
                "logo" => "arch",
                "completions" => "bash",
                "config" => "x.conf",
                _ => "ip",
            };
            assert!(
                parse_args(vec![format!("--{f}"), value.into()], Cli::default()).is_ok(),
                "--{f}"
            );
        }
        for f in hide_flags() {
            assert!(parse_args(vec![format!("--{f}")], Cli::default()).is_ok());
        }
    }

    #[test]
    fn scripts_cover_logos_modules_and_flags() {
        let b = script("bash").unwrap();
        assert!(b.contains("complete -F _rsi rsi") && b.contains("--no-ip"));
        assert!(b.contains("arch") && b.contains("--completions"));
        assert!(b.contains("--no-cache"));
        let z = script("zsh").unwrap();
        assert!(z.starts_with("#compdef rsi") && z.contains("'--no-gpu["));
        assert!(z.contains("--no-cache"));
        let f = script("fish").unwrap();
        assert!(f.contains("complete -c rsi -l logo") && f.contains("-l no-ip"));
        assert!(f.contains("-l no-cache"));
        assert!(script("tcsh").is_none());
    }
}
