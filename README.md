<div align="center">

# rsi — rustsysteminfo

Información del sistema con logo animado y monitor en vivo, en un único binario de Rust.
Una alternativa moderna a Neofetch y Fastfetch.

[![Release](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/release.yml/badge.svg)](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/release.yml)
[![Audit](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/audit.yml/badge.svg)](https://github.com/Linuxero-dev/rustsysteminfo/actions/workflows/audit.yml)
[![GitHub release](https://img.shields.io/github/v/release/Linuxero-dev/rustsysteminfo)](https://github.com/Linuxero-dev/rustsysteminfo/releases)
[![Licencia MIT](https://img.shields.io/badge/licencia-MIT-blue.svg)](LICENSE)

<img src="docs/rsi-pantalla-completa.png" alt="rsi mostrando la información del sistema en Konsole" width="900">

</div>

## Por qué rsi

- **En vivo**: CPU, RAM y swap se actualizan en pantalla, con el logo animado.
- **Completo**: WM, resolución con frecuencia de refresco, temperatura de CPU, VRAM de la GPU, paquetes, shell con versión, discos con barras y más.
- **Configurable**: oculta lo que no quieras (`--no-ip`), fija tus opciones en `~/.config/rsi/config` y elige el logo.
- **Para scripts**: `--json` imprime todos los datos en JSON.
- **Sin dependencias en tiempo de ejecución**: un solo binario estático para x86_64 y ARM64.
- **Seguro por diseño**: el instalador verifica el SHA256 y la procedencia, y rsi limpia de secuencias de escape todo texto que lee del sistema.
- **Se ve bien en cualquier terminal**: truecolor, 256 colores o 16, con `NO_COLOR` respetado y ancho correcto para CJK y emojis.

## Novedades en 0.5.0

- **Caché de paquetes y GPU** en `~/.cache/rsi`: el arranque en caliente pasa de ~880 ms a ~30-50 ms.
- **Corrección crítica de estabilidad**: los procesos huérfanos que consumían un núcleo completo al perder la terminal ahora salen limpiamente.
- **Recorte con elipsis** (`…`) cuando el texto no cabe en la terminal.
- **Anchos Unicode correctos** para símbolos (⌚⏰✅❌), emojis (🟠🩷🀄) y banderas.
- **Paleta de colores visible** con marco gris en lugar de cuadro negro invisible.

<details>
<summary>Más capturas</summary>

<img src="docs/rsi-ventana.jpg" alt="rsi en una ventana de Konsole sobre el escritorio" width="900">

</details>

## Instalación

### Instalación rápida (recomendada para la mayoría de usuarios)

Este método instala `rsi` en `/usr/local/bin`, disponible para todos los usuarios del sistema:

```bash
curl -sSL https://raw.githubusercontent.com/Linuxero-dev/rustsysteminfo/main/install.sh | sudo sh