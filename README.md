# rustsysteminfo (rsi)

Un clon moderno de Neofetch y Fastfetch escrito en Rust: logo animado, información del sistema, discos con barras y CPU/RAM en vivo.

## Instalación universal

Funciona en cualquier distribución Linux (x86_64 y ARM64) sin necesidad de tener Rust instalado.

```sh
curl -sSL https://raw.githubusercontent.com/Linuxero-dev/rustsysteminfo/main/install.sh | sh
```

El instalador comprueba siempre el SHA256 y, si tienes la CLI `gh`, también la procedencia del binario (ver [SECURITY.md](SECURITY.md)).

## Uso

```sh
rsi                    # monitor en vivo (Enter, Esc o q para salir)
rsi --once             # imprime una vez y sale
rsi --json             # datos en JSON, para scripts
rsi --no-ip            # oculta la IP local
rsi --hide ip,gpu      # oculta varios módulos
rsi --logo arch        # fuerza un logo
rsi --no-logo          # sin logo
```

| Opción | Descripción |
|---|---|
| `-h`, `--help` | muestra la ayuda |
| `-V`, `--version` | muestra la versión |
| `--once` | imprime una vez y sale |
| `--no-anim` | logo estático |
| `--json` | imprime los datos en JSON y sale |
| `--logo NOMBRE` | fuerza un logo (`--list-logos` los muestra) |
| `--no-logo` | no muestra el logo |
| `--no-MÓDULO` | oculta un módulo, por ejemplo `--no-ip` |
| `--hide A,B` | oculta varios módulos a la vez |
| `--show A,B` | muestra módulos que la configuración oculta |
| `--list-logos`, `--list-modules` | listan los logos y los módulos disponibles |
| `--config RUTA`, `--no-config` | usa otro fichero de configuración, o ninguno |
| `--gen-config` | imprime una configuración de ejemplo |
| `--completions SHELL` | imprime el completado para `bash`, `zsh` o `fish` |

Módulos: `os`, `host`, `kernel`, `uptime`, `init`, `processes`, `load`, `packages`, `shell`, `terminal`, `locale`, `ip`, `de`, `wm`, `display`, `theme`, `icons`, `font`, `cpu`, `temp`, `gpu`, `battery`, `disks`, `colors`.

La variable `NO_COLOR` desactiva los colores. Con terminales de 256 colores o *truecolor* se usa la mejor paleta disponible.

## Configuración

Para no repetir opciones, crea `~/.config/rsi/config` (o `$XDG_CONFIG_HOME/rsi/config`):

```sh
mkdir -p ~/.config/rsi
rsi --gen-config > ~/.config/rsi/config
```

```ini
# ~/.config/rsi/config
hide = ip,gpu
logo = arch
```

Claves: `hide`, `logo`, `no_logo` y `animate`. Las opciones de la línea de comandos tienen prioridad sobre el fichero (`--show ip` anula un `hide = ip`).

## Completado de comandos

```sh
# bash
mkdir -p ~/.local/share/bash-completion/completions
rsi --completions bash > ~/.local/share/bash-completion/completions/rsi

# zsh (con ~/.zfunc en el fpath)
rsi --completions zsh > ~/.zfunc/_rsi

# fish
rsi --completions fish > ~/.config/fish/completions/rsi.fish
```

## Notas

- La temperatura de la CPU se lee de `hwmon`; la VRAM, de sysfs (AMD) o de `nvidia-smi` (NVIDIA), y solo se muestra con una única GPU.
- La frecuencia de refresco sale del EDID de la pantalla, así que puede faltar en algunos monitores.
