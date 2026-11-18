# Cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [0.5.2] - 2026-11-17

### Añadido
- Paquete `.deb` (`amd64` y `arm64`) en cada release, con su SHA256 y su attestation de procedencia. Incluye el binario estático, la página de manual, el completado de bash, zsh y fish y la licencia, y no tiene dependencias.
- `scripts/make-deb.sh` y `scripts/test-deb.sh`; el CI y `check.sh` comprueban el paquete.
- El instalador avisa si ya hay un `.deb` de rsi instalado, y el `.deb` avisa si existe una copia en `/usr/local/bin`.
- README: instalación con `wget` y con `.deb`.

## [0.5.1] - 2026-11-17

### Añadido
- El instalador comprueba que `rsi` se ejecuta tras instalarlo (`rsi --version`), detecta si queda fuera del PATH o tapado por otro `rsi`, y da la línea exacta para bash, zsh o fish.
- El instalador usa `wget` cuando no hay `curl`, y explica qué paquete falta si no hay `tar` ni `sha256sum`.
- La desinstalación también borra la caché de `rsi`.

### Corregido
- `README.md` y `CHANGELOG.md` habían quedado truncados; se restauran.
- Ctrl+C durante la instalación interrumpe el instalador en lugar de continuar sin directorio temporal.
- Si `sudo` existe pero no se puede usar, el instalador cae a `~/.local/bin` en vez de fallar.
- Caché de paquetes obsoleta en NixOS y Gentoo.
- Si el modo interactivo no arranca, Ctrl+C conserva su comportamiento normal.

### Rendimiento
- La tabla de colores del logo se calcula una sola vez, y el monitor refresca solo el uso de CPU.

### Seguridad
- La caché usa un directorio `0700`, un fichero temporal único `0600` creado en exclusiva.

## [0.5.0] - 2026-11-17

### Añadido
- Caché en `~/.cache/rsi` para el conteo de paquetes y el nombre de la GPU,
  invalidada automáticamente al cambiar el sistema o al reiniciar.
- Opción `--no-cache` para recalcular todo ignorando la caché.
- Muestreo de CPU sin espera fija: reutiliza la muestra anterior si es reciente.

### Corregido
- **Procesos huérfanos que consumían un núcleo completo.** Si `rsi` perdía su
  terminal de forma brusca (cierre del emulador, caída de una sesión SSH, o un
  script que lo lanza y muere sin esperarlo), el proceso quedaba vivo sin
  terminal y entraba en un bucle que ocupaba entre el 60 % y el 90 % de un
  núcleo de forma permanente. Ahora detecta que la terminal desapareció, tanto
  por fallo del ioctl de tamaño como por fallo al escribir, y sale limpiando la
  pantalla. Además el bucle principal nunca espera menos de 1 ms por vuelta, de
  modo que ningún fallo puede degenerar en un giro a máxima velocidad.
- Los textos que no caben en el ancho de la terminal se recortan con `…` en
  lugar de cortarse a mitad de carácter.
- Ancho correcto en terminal para símbolos y emojis que se medían como una
  columna: `⌚ ⏰ ✅ ❌ 🟠 🩷 🀄`, banderas y caracteres invisibles.
- La paleta de colores ya no tiene un cuadro negro invisible sobre fondos
  oscuros: lleva un marco gris que delimita cada muestra.

### Rendimiento
- El arranque en caliente pasa de ~880 ms a ~30-50 ms gracias a la caché.
- El modo en vivo repinta solo las filas que cambian y emite menos códigos de
  color: de ~3,2 % de un núcleo y ~250 KiB/s a cerca del 1 % y menos de 80 KiB/s.

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

[0.5.2]: https://github.com/Linuxero-dev/rustsysteminfo/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/Linuxero-dev/rustsysteminfo/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Linuxero-dev/rustsysteminfo/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/Linuxero-dev/rustsysteminfo/releases/tag/v0.4.2
