#!/usr/bin/env bash
# Crea el paquete .deb de rsi a partir de un binario ya compilado (estático, sin dependencias).
#
# Uso: scripts/make-deb.sh BINARIO ARQUITECTURA VERSIÓN DIRECTORIO_SALIDA [BINARIO_EJECUTABLE]
#   BINARIO             binario de rsi que irá dentro del paquete (puede ser de otra arquitectura)
#   ARQUITECTURA        amd64 | arm64
#   VERSIÓN             p. ej. 0.5.2
#   DIRECTORIO_SALIDA   dónde se escribe rustsysteminfo_<versión>_<arquitectura>.deb
#   BINARIO_EJECUTABLE  binario que se pueda ejecutar en esta máquina, para generar el
#                       completado (por defecto, BINARIO). Si no se puede ejecutar, el
#                       paquete se crea sin completado.
set -euo pipefail
umask 022

NAME="rustsysteminfo"
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"

usage() {
    echo "Uso: $0 BINARIO ARQUITECTURA(amd64|arm64) VERSIÓN DIRECTORIO_SALIDA [BINARIO_EJECUTABLE]" >&2
    exit 2
}

[ $# -ge 4 ] && [ $# -le 5 ] || usage
BIN=$1
ARCH=$2
VERSION=$3
OUT=$4
RUNNER=${5:-$1}

case "$ARCH" in
    amd64 | arm64) ;;
    *) echo "Error: arquitectura no válida: $ARCH (usa amd64 o arm64)" >&2; exit 2 ;;
esac
[[ $VERSION =~ ^[0-9][0-9A-Za-z.+~-]*$ ]] || { echo "Error: versión no válida: $VERSION" >&2; exit 2; }
[ -f "$BIN" ] || { echo "Error: no existe el binario $BIN" >&2; exit 1; }
command -v dpkg-deb >/dev/null 2>&1 || { echo "Error: se necesita dpkg-deb" >&2; exit 1; }
for f in man/rsi.1 LICENSE; do
    [ -f "$ROOT/$f" ] || { echo "Error: falta $f" >&2; exit 1; }
done

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
PKG="$STAGE/pkg"

install -D -m 755 "$BIN" "$PKG/usr/bin/rsi"
install -d -m 755 "$PKG/usr/share/man/man1"
gzip -9n -c "$ROOT/man/rsi.1" >"$PKG/usr/share/man/man1/rsi.1.gz"
chmod 644 "$PKG/usr/share/man/man1/rsi.1.gz"

# Completado de comandos: lo genera el propio binario.
if "$RUNNER" --version >/dev/null 2>&1; then
    install -d -m 755 \
        "$PKG/usr/share/bash-completion/completions" \
        "$PKG/usr/share/zsh/vendor-completions" \
        "$PKG/usr/share/fish/vendor_completions.d"
    "$RUNNER" --completions bash >"$PKG/usr/share/bash-completion/completions/rsi"
    "$RUNNER" --completions zsh >"$PKG/usr/share/zsh/vendor-completions/_rsi"
    "$RUNNER" --completions fish >"$PKG/usr/share/fish/vendor_completions.d/rsi.fish"
    chmod 644 "$PKG"/usr/share/bash-completion/completions/rsi \
        "$PKG"/usr/share/zsh/vendor-completions/_rsi \
        "$PKG"/usr/share/fish/vendor_completions.d/rsi.fish
else
    echo "Aviso: $RUNNER no se puede ejecutar aquí; el paquete se crea sin completado." >&2
fi

DOC="$PKG/usr/share/doc/$NAME"
install -d -m 755 "$DOC"
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: rustsysteminfo"
    echo "Source: https://github.com/Linuxero-dev/rustsysteminfo"
    echo "License: MIT"
    echo
    cat "$ROOT/LICENSE"
} >"$DOC/copyright"
chmod 644 "$DOC/copyright"

SIZE="$(du -sk "$PKG/usr" | cut -f1)"

install -d -m 755 "$PKG/DEBIAN"
cat >"$PKG/DEBIAN/control" <<EOF
Package: $NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Installed-Size: $SIZE
Maintainer: Linuxero-dev
Homepage: https://github.com/Linuxero-dev/rustsysteminfo
Description: información del sistema con logo animado y monitor en vivo
 rsi es una alternativa moderna a Neofetch y Fastfetch: muestra los datos
 del sistema junto a un logo animado y mide CPU, RAM y swap en vivo.
 .
 Es un binario estático de Rust, sin dependencias en tiempo de ejecución.
EOF

# Si alguien instaló antes rsi con el script (curl), esa copia en /usr/local/bin
# tapará a la del paquete: se avisa, sin tocar nada.
cat >"$PKG/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = "configure" ] && [ -x /usr/local/bin/rsi ]; then
    echo "Aviso: existe /usr/local/bin/rsi (instalado con el script) y tiene prioridad sobre /usr/bin/rsi." >&2
    echo "       Para usar la del paquete: sudo rm /usr/local/bin/rsi" >&2
fi
exit 0
EOF
chmod 755 "$PKG/DEBIAN/postinst"

mkdir -p "$OUT"
DEB="$OUT/${NAME}_${VERSION}_${ARCH}.deb"
# xz en vez de zstd: lo entienden también Debian y Ubuntu antiguos.
dpkg-deb --root-owner-group -Zxz --build "$PKG" "$DEB" >/dev/null
echo "$DEB"
