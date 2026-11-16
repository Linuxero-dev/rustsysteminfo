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
  
[0.5.0]: https://github.com/Linuxero-dev/rustsysteminfo/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/Linuxero-dev/rustsysteminfo/releases/tag/v0.4.2