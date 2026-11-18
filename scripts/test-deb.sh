#!/usr/bin/env bash
# Comprueba que scripts/make-deb.sh genera un .deb correcto.
# Uso: scripts/test-deb.sh [BINARIO]   (por defecto target/release/rsi)
set -uo pipefail
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
BIN="${1:-$ROOT/target/release/rsi}"
[ -x "$BIN" ] || { echo "No existe $BIN: ejecuta antes 'cargo build --release'." >&2; exit 1; }
command -v dpkg-deb >/dev/null 2>&1 || { echo "dpkg-deb no está instalado: se omiten las pruebas del .deb."; exit 0; }

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

echo "make-deb.sh"
for arch in amd64 arm64; do
    "$ROOT/scripts/make-deb.sh" "$BIN" "$arch" "$VERSION" "$WORK/out" "$BIN" >/dev/null 2>&1
    deb="$WORK/out/rustsysteminfo_${VERSION}_${arch}.deb"
    check "crea el paquete para $arch" test -f "$deb"
    check "$arch: campo Package" test "$(dpkg-deb -f "$deb" Package)" = "rustsysteminfo"
    check "$arch: campo Version" test "$(dpkg-deb -f "$deb" Version)" = "$VERSION"
    check "$arch: campo Architecture" test "$(dpkg-deb -f "$deb" Architecture)" = "$arch"
    check "$arch: sin dependencias" test -z "$(dpkg-deb -f "$deb" Depends)"
done

deb="$WORK/out/rustsysteminfo_${VERSION}_amd64.deb"
mkdir "$WORK/x"
dpkg-deb -x "$deb" "$WORK/x"
check "instala el binario en /usr/bin/rsi" test -x "$WORK/x/usr/bin/rsi"
check "el binario del paquete es el original" cmp -s "$BIN" "$WORK/x/usr/bin/rsi"
check "instala la página de manual comprimida" gzip -t "$WORK/x/usr/share/man/man1/rsi.1.gz"
check "instala el completado de bash" test -s "$WORK/x/usr/share/bash-completion/completions/rsi"
check "instala el completado de zsh" test -s "$WORK/x/usr/share/zsh/vendor-completions/_rsi"
check "instala el completado de fish" test -s "$WORK/x/usr/share/fish/vendor_completions.d/rsi.fish"
check "incluye el copyright con la licencia" grep -q 'MIT' "$WORK/x/usr/share/doc/rustsysteminfo/copyright"
check "no instala nada fuera de /usr" test "$(ls -A "$WORK/x")" = "usr"
check "nada del paquete es escribible por otros" \
    test -z "$(find "$WORK/x" -perm /o+w)"
check "los ficheros pertenecen a root" \
    not bash -c "dpkg-deb -c '$deb' | awk '{print \$2}' | grep -qv '^root/root$'"

mkdir "$WORK/c"
dpkg-deb -e "$deb" "$WORK/c"
check "tiene un postinst ejecutable" test -x "$WORK/c/postinst"
check "el postinst es sintácticamente válido" sh -n "$WORK/c/postinst"

check "rechaza una arquitectura no válida" \
    not "$ROOT/scripts/make-deb.sh" "$BIN" i386 "$VERSION" "$WORK/mal"
check "rechaza una versión no válida" \
    not "$ROOT/scripts/make-deb.sh" "$BIN" amd64 "1;rm" "$WORK/mal"

printf '\177ELF-roto' >"$WORK/roto"
chmod +x "$WORK/roto"
"$ROOT/scripts/make-deb.sh" "$BIN" arm64 "$VERSION" "$WORK/sc" "$WORK/roto" >/dev/null 2>&1
check "sin binario ejecutable crea el paquete sin completado" \
    test -f "$WORK/sc/rustsysteminfo_${VERSION}_arm64.deb"
check "sin binario ejecutable no incluye completado" \
    not bash -c "dpkg-deb -c '$WORK/sc/rustsysteminfo_${VERSION}_arm64.deb' | grep -q completion"

if [ "$fails" -ne 0 ]; then
    echo "$fails prueba(s) fallida(s)"
    exit 1
fi
echo "Todas las pruebas del .deb pasaron."
