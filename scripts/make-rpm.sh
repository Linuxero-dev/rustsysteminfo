#!/usr/bin/env bash
# Crea el paquete .rpm de rsi a partir de un binario ya compilado (estático, sin dependencias).
#
# Uso: scripts/make-rpm.sh BINARIO ARQUITECTURA VERSIÓN DIRECTORIO_SALIDA [BINARIO_EJECUTABLE]
#   BINARIO             binario de rsi que irá dentro del paquete (puede ser de otra arquitectura)
#   ARQUITECTURA        x86_64 | aarch64
#   VERSIÓN             p. ej. 0.5.3
#   DIRECTORIO_SALIDA   dónde se escribe rustsysteminfo-<versión>-1.<arquitectura>.rpm
#   BINARIO_EJECUTABLE  binario que se pueda ejecutar en esta máquina, para generar el
#                       completado (por defecto, BINARIO). Si no se puede ejecutar, el
#                       paquete se crea sin completado.
set -euo pipefail
umask 022

NAME="rustsysteminfo"
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"

usage() {
    echo "Uso: $0 BINARIO ARQUITECTURA(x86_64|aarch64) VERSIÓN DIRECTORIO_SALIDA [BINARIO_EJECUTABLE]" >&2
    exit 2
}

if [ "$#" -lt 4 ] || [ "$#" -gt 5 ]; then usage; fi
BIN=$1
ARCH=$2
VERSION=$3
OUT=$4
RUNNER=${5:-$1}

case "$ARCH" in
    x86_64 | aarch64) ;;
    *) echo "Error: arquitectura no válida: $ARCH (usa x86_64 o aarch64)" >&2; exit 2 ;;
esac
# En RPM la versión no admite guiones.
[[ $VERSION =~ ^[0-9][0-9A-Za-z.+]*$ ]] || { echo "Error: versión no válida: $VERSION" >&2; exit 2; }
[ -f "$BIN" ] || { echo "Error: no existe el binario $BIN" >&2; exit 1; }
command -v rpmbuild >/dev/null 2>&1 || { echo "Error: se necesita rpmbuild (paquete rpm)" >&2; exit 1; }
for f in man/rsi.1 LICENSE packaging/rustsysteminfo.spec; do
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
        "$PKG/usr/share/zsh/site-functions" \
        "$PKG/usr/share/fish/vendor_completions.d"
    "$RUNNER" --completions bash >"$PKG/usr/share/bash-completion/completions/rsi"
    "$RUNNER" --completions zsh >"$PKG/usr/share/zsh/site-functions/_rsi"
    "$RUNNER" --completions fish >"$PKG/usr/share/fish/vendor_completions.d/rsi.fish"
    chmod 644 "$PKG"/usr/share/bash-completion/completions/rsi \
        "$PKG"/usr/share/zsh/site-functions/_rsi \
        "$PKG"/usr/share/fish/vendor_completions.d/rsi.fish
else
    echo "Aviso: $RUNNER no se puede ejecutar aquí; el paquete se crea sin completado." >&2
fi

install -D -m 644 "$ROOT/LICENSE" "$PKG/usr/share/licenses/$NAME/LICENSE"

mkdir -p "$OUT" "$STAGE/top"
OUT="$(cd "$OUT" && pwd)"

# El spec lista los ficheros de completado: si falta alguno (sin binario ejecutable),
# se crean vacíos para que el paquete siga siendo válido.
for f in usr/share/bash-completion/completions/rsi usr/share/zsh/site-functions/_rsi usr/share/fish/vendor_completions.d/rsi.fish; do
    if [ ! -f "$PKG/$f" ]; then
        install -D -m 644 /dev/null "$PKG/$f"
    fi
done

# xz en vez de zstd: lo entienden también los rpm antiguos.
rpmbuild -bb "$ROOT/packaging/rustsysteminfo.spec" \
    --target "$ARCH" \
    --define "_topdir $STAGE/top" \
    --define "_rpmdir $OUT" \
    --define "_rpmfilename %%{NAME}-%%{VERSION}-%%{RELEASE}.%%{ARCH}.rpm" \
    --define "_binary_payload w9.xzdio" \
    --define "rsi_version $VERSION" \
    --define "rsi_stage $PKG" >"$STAGE/rpmbuild.log" 2>&1 || {
    cat "$STAGE/rpmbuild.log" >&2
    exit 1
}
echo "$OUT/${NAME}-${VERSION}-1.${ARCH}.rpm"
