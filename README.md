<div align="center">

# rsi — rustsysteminfo

**Información del sistema con logo animado y monitor en vivo, en un único binario de Rust.**
Una alternativa moderna a Neofetch y Fastfetch.

[![Release](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/release.yml/badge.svg)](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/release.yml)
[![Audit](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/audit.yml/badge.svg)](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/audit.yml)
[![Última versión](https://img.shields.io/github/v/release/Linuxero-dev/rustsysteminfo)](https://github.com/Linuxero-dev/rustsysteminfo/releases/latest)
[![Licencia: MIT](https://img.shields.io/badge/licencia-MIT-blue.svg)](LICENSE)

<img src="docs/rsi-pantalla-completa.png" alt="rsi mostrando la información del sistema en Konsole" width="900">

</div>

## Por qué rsi

- **En vivo:** CPU, RAM y swap se actualizan en pantalla, con el logo animado.
- **Completo:** WM, resolución con frecuencia de refresco, temperatura de CPU, VRAM de la GPU, paquetes, shell con versión, discos con barras y más.
- **Configurable:** oculta lo que no quieras (`--no-ip`), fija tus opciones en `~/.config/rsi/config` y elige el logo.
- **Para scripts:** `--json` imprime todos los datos en JSON.
- **Sin dependencias en tiempo de ejecución:** un solo binario estático para x86_64 y ARM64.
- **Seguro por diseño:** el instalador verifica el SHA256 y la procedencia, y rsi limpia de secuencias de escape todo texto que lee del sistema.
- **Se ve bien en cualquier terminal:** truecolor, 256 colores o 16, con `NO_COLOR` respetado y ancho correcto para CJK y emojis.

<details>
<summary>Más capturas</summary>

<img src="docs/rsi-ventana.jpg" alt="rsi en una ventana de Konsole sobre el escritorio" width="900">

</details>

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
