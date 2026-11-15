# rustsysteminfo (rsi)

Un clon moderno de Neofetch y Fastfetch escrito en Rust: logo animado, discos con barras y CPU/RAM en vivo.

## Instalación universal

Funciona en cualquier distribución Linux (x86_64 y ARM64) sin necesidad de tener Rust instalado.

```sh
curl -sSL https://raw.githubusercontent.com/Linuxero-dev/rustsysteminfo/main/install.sh | sh
```

## Uso

```sh
rsi                    # monitor en vivo (Enter, Esc o q para salir)
rsi --once             # imprime una vez y sale
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
| `--logo NOMBRE` | fuerza un logo (`--list-logos` los muestra) |
| `--no-logo` | no muestra el logo |
| `--no-MÓDULO` | oculta un módulo, por ejemplo `--no-ip` |
| `--hide A,B` | oculta varios módulos a la vez |
| `--list-logos`, `--list-modules` | listan los logos y los módulos disponibles |

Módulos: `os`, `host`, `kernel`, `uptime`, `init`, `processes`, `load`, `packages`, `shell`, `terminal`, `locale`, `ip`, `de`, `display`, `theme`, `icons`, `font`, `cpu`, `gpu`, `battery`, `disks`, `colors`.

La variable `NO_COLOR` desactiva los colores.
