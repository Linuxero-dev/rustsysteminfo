#!/usr/bin/env bash
# Pruebas de install.sh con un curl simulado (no usa la red ni toca tu HOME real).
set -uo pipefail
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$WORK/stub" "$WORK/home/.local/bin"
cat >"$WORK/stub/curl" <<'STUB'
#!/bin/sh
# Simula curl: genera un paquete falso según STUB_MODE (ok | bad-checksum | no-binary | broken-bin | slow).
url=""; out=""
while [ $# -gt 0 ]; do
    case "$1" in
    -o) out="$2"; shift 2 ;;
    --proto | --retry | --connect-timeout | --max-time) shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
    esac
done
[ "${STUB_MODE:-ok}" = "slow" ] && sleep 2
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
    elif [ "${STUB_MODE:-ok}" = "broken-bin" ]; then
        printf '#!/bin/sh\necho roto >&2\nexit 1\n' >"$d/rsi"
        tar -czf "$out" -C "$d" rsi
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

# Simula wget (-q -O fichero url) reutilizando el curl simulado.
cat >"$WORK/stub/wget" <<STUB
#!/bin/sh
[ "\$1" = "-q" ] && [ "\$2" = "-O" ] || exit 2
exec "$WORK/stub/curl" "\$4" -o "\$3"
STUB
chmod +x "$WORK/stub/wget"

# Simula sudo: "-v" responde según SUDO_V (0 = utilizable); el resto se ejecuta sin elevación.
cat >"$WORK/stub/sudo" <<'STUB'
#!/bin/sh
if [ "${1:-}" = "-v" ]; then exit "${SUDO_V:-0}"; fi
exec "$@"
STUB
chmod +x "$WORK/stub/sudo"

# PATH mínimo sin curl ni wget, para probar los mensajes de herramientas ausentes.
mkdir -p "$WORK/bare"
for t in tar mktemp uname basename dirname install mv rm mkdir sha256sum cat gzip sleep; do
    p=$(command -v "$t" 2>/dev/null) && ln -sf "$p" "$WORK/bare/$t"
done

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
# PATH aislado (solo los simulados y las herramientas básicas): así el resultado no depende
# de que el equipo donde se prueba ya tenga un "rsi" instalado.
install_out() { HOME="$WORK/home" PATH="$WORK/stub:$WORK/bare" RSI_VERIFY_ATTESTATION=0 /bin/sh "$ROOT/install.sh" "$@" >"$WORK/out.txt" 2>&1; }
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
STUB_MODE=ok HOME="$WORK/home" PATH="$WORK/stub:$WORK/bare" RSI_VERIFY_ATTESTATION=0 \
    /bin/sh "$ROOT/install.sh" ./hint >"$WORK/hint.out" 2>&1 || true
check "explica cómo usar rsi si el destino no está en el PATH" grep -q 'export PATH' "$WORK/hint.out"
check "rechaza una RSI_VERSION con caracteres raros" \
    not env RSI_VERSION='v1;rm -rf /' PATH="$WORK/stub:$PATH" sh "$ROOT/install.sh" ./x

# --- comprobaciones posteriores a la instalación -------------------------------------------
STUB_MODE=broken-bin install_out ./brk
check "falla con un mensaje claro si el binario instalado no se ejecuta" grep -q 'no se puede ejecutar' "$WORK/out.txt"
check "el instalador devuelve error si el binario no se ejecuta" not env STUB_MODE=broken-bin HOME="$WORK/home" PATH="$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION=0 sh "$ROOT/install.sh" ./brk2

mkdir -p "$WORK/otro"
printf '#!/bin/sh\necho otro\n' >"$WORK/otro/rsi"
chmod +x "$WORK/otro/rsi"
STUB_MODE=ok HOME="$WORK/home" PATH="$WORK/otro:$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION=0 \
    sh "$ROOT/install.sh" ./sombra >"$WORK/out.txt" 2>&1
check "avisa si otro 'rsi' del PATH tapa al instalado" grep -q 'otro programa' "$WORK/out.txt"

STUB_MODE=ok SHELL=/usr/bin/fish install_out ./fishy
check "el aviso de PATH usa fish_add_path con fish" grep -q 'fish_add_path' "$WORK/out.txt"
STUB_MODE=ok SHELL=/usr/bin/zsh install_out ./zshy
check "el aviso de PATH usa ~/.zshrc con zsh" grep -q '.zshrc' "$WORK/out.txt"

# --- descargador alternativo y herramientas ausentes ---------------------------------------
mkdir -p "$WORK/solowget"
cp "$WORK"/bare/* "$WORK/solowget/"
ln -sf "$WORK/stub/wget" "$WORK/solowget/wget"
STUB_MODE=ok HOME="$WORK/home" PATH="$WORK/solowget" RSI_VERIFY_ATTESTATION=0 /bin/sh "$ROOT/install.sh" ./conwget >/dev/null 2>&1
check "usa wget cuando no hay curl" test -x "$WORK/cwd/conwget/rsi"
HOME="$WORK/home" PATH="$WORK/bare" /bin/sh "$ROOT/install.sh" ./nada >"$WORK/out.txt" 2>&1
check "explica que falta curl o wget" grep -q 'curl o wget' "$WORK/out.txt"

# --- la instalación se interrumpe con TERM y no deja temporales ----------------------------
mkdir -p "$WORK/tmpd"
STUB_MODE=slow TMPDIR="$WORK/tmpd" HOME="$WORK/home" PATH="$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION=0 \
    sh "$ROOT/install.sh" ./interrumpido >/dev/null 2>&1 &
pid=$!
sleep 0.5
kill -TERM "$pid" 2>/dev/null
wait "$pid" 2>/dev/null
check "una interrupción (TERM) no instala nada" not test -e "$WORK/cwd/interrumpido/rsi"
check "una interrupción (TERM) limpia el directorio temporal" test -z "$(ls -A "$WORK/tmpd")"

# --- la desinstalación borra la caché ------------------------------------------------------
mkdir -p "$WORK/cache/rsi"
echo x >"$WORK/cache/rsi/cache.tsv"
STUB_MODE=ok install_sh ./conc
XDG_CACHE_HOME="$WORK/cache" install_sh --uninstall ./conc
check "--uninstall borra la caché de rsi" not test -e "$WORK/cache/rsi"

# --- destino por defecto con sudo inutilizable (solo si no somos root) ---------------------
if [ ! -w /usr/local/bin ]; then
    SUDO_V=1 STUB_MODE=ok install_out
    check "sin sudo utilizable cae a ~/.local/bin" test -x "$WORK/home/.local/bin/rsi"
    install_sh --uninstall --local
    SUDO_V=1 STUB_MODE=ok env -u HOME PATH="$WORK/stub:$PATH" RSI_VERIFY_ATTESTATION=0 \
        sh "$ROOT/install.sh" >"$WORK/out.txt" 2>&1
    check "sin sudo ni HOME falla con un mensaje claro" grep -q 'HOME' "$WORK/out.txt"
fi

if [ "$fails" -ne 0 ]; then
    echo "$fails prueba(s) fallida(s)"
    exit 1
fi
echo "Todas las pruebas del instalador pasaron."