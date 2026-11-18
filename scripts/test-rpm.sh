#!/usr/bin/env bash
# Comprueba que scripts/make-rpm.sh genera un .rpm correcto.
# Uso: scripts/test-rpm.sh [BINARIO]   (por defecto target/release/rsi)
set -uo pipefail
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
BIN="${1:-$ROOT/target/release/rsi}"
[ -x "$BIN" ] || { echo "No existe $BIN: ejecuta antes 'cargo build --release'." >&2; exit 1; }
if ! command -v rpmbuild >/dev/null 2>&1 || ! command -v rpm >/dev/null 2>&1; then
    echo "rpm/rpmbuild no están instalados: se omiten las pruebas del .rpm."
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)"

fails=0
check() {
    local desc="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        echo "  ok    $desc"
    else
        echo "  FALLO $desc"
        fails=$((fails + 1))
    fi
}
not() { ! "$@"; }
query() { rpm -qp --qf "$2" "$1" 2>/dev/null; }

echo "make-rpm.sh"
for arch in x86_64 aarch64; do
    "$ROOT/scripts/make-rpm.sh" "$BIN" "$arch" "$VERSION" "$WORK/out" "$BIN" >"$WORK/log.txt" 2>&1 || cat "$WORK/log.txt"
    rpmf="$WORK/out/rustsysteminfo-${VERSION}-1.${arch}.rpm"
    check "crea el paquete para $arch" test -f "$rpmf"
    check "$arch: campo Name" test "$(query "$rpmf" '%{NAME}')" = "rustsysteminfo"
    check "$arch: campo Version" test "$(query "$rpmf" '%{VERSION}')" = "$VERSION"
    check "$arch: campo Arch" test "$(query "$rpmf" '%{ARCH}')" = "$arch"
    check "$arch: licencia MIT" test "$(query "$rpmf" '%{LICENSE}')" = "MIT"
    check "$arch: sin dependencias salvo las internas de rpm" \
        test -z "$(rpm -qp --requires "$rpmf" 2>/dev/null | grep -v -e '^rpmlib(' -e '^/bin/sh')"
done

rpmf="$WORK/out/rustsysteminfo-${VERSION}-1.x86_64.rpm"
files="$(rpm -qpl "$rpmf" 2>/dev/null)"
has() { printf '%s\n' "$files" | grep -qx "$1"; }
check "instala el binario en /usr/bin/rsi" has /usr/bin/rsi
check "instala la página de manual" bash -c "printf '%s\n' \"\$1\" | grep -q '^/usr/share/man/man1/rsi\.1'" _ "$files"
check "instala el completado de bash" has /usr/share/bash-completion/completions/rsi
check "instala el completado de zsh" has /usr/share/zsh/site-functions/_rsi
check "instala el completado de fish" has /usr/share/fish/vendor_completions.d/rsi.fish
check "marca la licencia como tal" bash -c "rpm -qpL '$rpmf' | grep -qx /usr/share/licenses/rustsysteminfo/LICENSE"
check "el binario es ejecutable y de root" \
    bash -c "rpm -qp --qf '[%{FILEMODES:perms} %{FILEUSERNAME}:%{FILEGROUPNAME} %{FILENAMES}\n]' '$rpmf' | grep -qx -- '-rwxr-xr-x root:root /usr/bin/rsi'"
check "no instala nada fuera de /usr" test -z "$(printf '%s\n' "$files" | grep -v '^/usr/')"

check "rechaza una arquitectura no válida" \
    not "$ROOT/scripts/make-rpm.sh" "$BIN" i686 "$VERSION" "$WORK/mal"
check "rechaza una versión con guion" \
    not "$ROOT/scripts/make-rpm.sh" "$BIN" x86_64 "1.0-rc1" "$WORK/mal"
check "rechaza una versión con caracteres raros" \
    not "$ROOT/scripts/make-rpm.sh" "$BIN" x86_64 "1;rm" "$WORK/mal"

printf '\177ELF-roto' >"$WORK/roto"
chmod +x "$WORK/roto"
"$ROOT/scripts/make-rpm.sh" "$BIN" aarch64 "$VERSION" "$WORK/sc" "$WORK/roto" >/dev/null 2>&1
check "sin binario ejecutable crea igualmente el paquete" \
    test -f "$WORK/sc/rustsysteminfo-${VERSION}-1.aarch64.rpm"

if [ "$fails" -ne 0 ]; then
    echo "$fails prueba(s) fallida(s)"
    exit 1
fi
echo "Todas las pruebas del .rpm pasaron."
