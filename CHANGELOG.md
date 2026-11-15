# Cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [0.4.2]
### Corregido
- El instalador explica cómo ejecutar `rsi` cuando el destino no está en el PATH (caso típico de Ubuntu justo tras instalar).
### Cambiado
- La ola del logo es más lenta y suave.

## [0.4.1]
### Corregido
- Las líneas más anchas que la terminal ya no pierden los colores al recortarse.

## [0.4.0]
### Añadido
- Módulos nuevos: WM, resolución con frecuencia de refresco, versión del shell, temperatura de CPU y VRAM de la GPU. Núcleos/hilos siempre visibles.
- Fichero de configuración `~/.config/rsi/config` (`hide`, `logo`, `no_logo`, `animate`), con `--config`, `--no-config`, `--show` y `--gen-config`.
- Salida `--json`.
- Completado de comandos para bash, zsh y fish (`--completions`).
- Paleta de 256 colores y ancho real de caracteres (CJK, emojis y acentos combinados).
- Títulos de sección con línea decorativa.
- El instalador verifica la procedencia con `gh` cuando está instalado (`RSI_VERIFY_ATTESTATION`).

## [0.3.0]
### Añadido
- Opciones `-V/--version`, `--logo`, `--no-logo`, `--list-logos`, `--no-MÓDULO`, `--hide` y `--list-modules`.
- Módulo de IP local.
### Corregido
- El monitor ya no redibuja con cada evento del ratón, lo que disparaba el uso de CPU al girar la rueda.

## [0.2.5]
### Corregido
- Restos de líneas al cambiar entre layouts en el monitor en vivo.
- Bloque de código sin cerrar en el README.
