#!/usr/bin/env bash
# Pruebas de install.sh con un curl simulado (no usa la red ni toca tu HOME real).
set -uo pipefail
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$WORK/stub" "$WORK/home/.local/bin"
cat >"$WORK/stub/curl" <<'STUB'
#!/bin/sh
# Simula curl: genera un paquete falso según STUB_MODE (ok | bad-checksum | no-binary).
url=""; out=""
while [ $# -gt 0 ]; do
    case "$1" in
    -o) out="$2"; shift 2 ;;
    --proto | --retry | --connect-timeout | --max-time) shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
    esac
done
case "$url" in
*.sha256)
    f="${out%.sha256}"
    if [ "${STUB_MODE:-ok}" = "bad-checksum" ]; then
        echo "0000000000000000000000000000000000000000000000000000000000000000  $(basename "$f")" >"$out"
    else
        (cd "$(dirname "$out")" && sha256sum "$(basename "$f")" >"$out")
    fi
    ;;
*.tar.gz)
    d=$(mktemp -d)
    if [ "${STUB_MODE:-ok}" = "no-binary" ]; then
        echo x >"$d/otro-fichero"
        tar -czf "$out" -C "$d" otro-fichero
    else
        printf '#!/bin/sh\necho rsi-falso\n' >"$d/rsi"
        echo '.TH RSI 1' >"$d/rsi.1"
        tar -czf "$out" -C "$d" rsi rsi.1
    fi
    rm -rf "$d"
    ;;
*) exit 22 ;;
esac
STUB
chmod +x "$WORK/stub/curl"

# Simula gh: "gh attestation verify" tiene éxito o falla según GH_MODE (ok | fail).
cat >"$WORK/stub/gh" <<'STUB'
#!/bin/sh
[ "${GH_MODE:-ok}" = "ok" ]
STUB
chmod +x "$WORK/stub/gh"

# Simula sudo: simplemente ejecuta el comando sin elevación.
cat >"$WORK/stub/sudo" <<'STUB'
#!/bin/sh
exec "$@"
STUB
chmod +x "$WORK/stub/sudo"

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
install_sh() { HOME="$WORK/home" PATH="$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION="${ATT:-0}" sh "$ROOT/install.sh" "$@" >/dev/null 2>&1; }
not() { ! "$@"; }

echo "install.sh"
mkdir -p "$WORK/cwd"
cd "$WORK/cwd" || exit 1

STUB_MODE=ok install_sh ./bin
check "instala en una ruta relativa (./bin)" test -x "$WORK/cwd/bin/rsi"
check "instala la página de manual junto a <prefijo>/bin" test -f "$WORK/cwd/share/man/man1/rsi.1"
check "no deja ficheros temporales en el destino" test "$(ls -A "$WORK/cwd/bin")" = "rsi"

STUB_MODE=ok install_sh --local
check "--local instala en ~/.local/bin" test -x "$WORK/home/.local/bin/rsi"

install_sh --uninstall --local
check "--uninstall --local elimina el binario" not test -e "$WORK/home/.local/bin/rsi"

install_sh --uninstall ./bin
check "--uninstall con directorio relativo" not test -e "$WORK/cwd/bin/rsi"
check "--uninstall elimina la página de manual" not test -e "$WORK/cwd/share/man/man1/rsi.1"

STUB_MODE=bad-checksum install_sh ./otro
check "rechaza una suma SHA256 incorrecta" not test -e "$WORK/cwd/otro/rsi"
STUB_MODE=no-binary install_sh ./otro2
check "rechaza un paquete sin el binario" not test -e "$WORK/cwd/otro2/rsi"
check "rechaza opciones desconocidas" not sh "$ROOT/install.sh" --nope
GH_MODE=ok ATT=auto STUB_MODE=ok install_sh ./att1
check "procedencia auto: instala si gh verifica" test -x "$WORK/cwd/att1/rsi"
GH_MODE=fail ATT=auto STUB_MODE=ok install_sh ./att2
check "procedencia auto: instala con aviso si gh no puede verificar" test -x "$WORK/cwd/att2/rsi"
GH_MODE=ok ATT=1 STUB_MODE=ok install_sh ./att3
check "procedencia=1: instala si gh verifica" test -x "$WORK/cwd/att3/rsi"
GH_MODE=fail ATT=1 STUB_MODE=ok install_sh ./att4
check "procedencia=1: rechaza si la verificación falla" not test -e "$WORK/cwd/att4/rsi"
STUB_MODE=ok ATT=quizas install_sh ./att5
check "rechaza un valor raro de RSI_VERIFY_ATTESTATION" not test -e "$WORK/cwd/att5/rsi"
STUB_MODE=ok HOME="$WORK/home" PATH="$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION=0 \
    sh "$ROOT/install.sh" ./hint >"$WORK/hint.out" 2>&1 || true
check "explica cómo usar rsi si el destino no está en el PATH" grep -q 'export PATH' "$WORK/hint.out"
check "rechaza una RSI_VERSION con caracteres raros" \
    not env RSI_VERSION='v1;rm -rf /' PATH="$WORK/stub:$PATH" sh "$ROOT/install.sh" ./x

if [ "$fails" -ne 0 ]; then
    echo "$fails prueba(s) fallida(s)"
    exit 1
fi
echo "Todas las pruebas del instalador pasaron."