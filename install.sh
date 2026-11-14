#!/bin/sh
# Instalador de rsi (rustsysteminfo).
# Descarga el binario del último release de GitHub, verifica su SHA256 y lo instala.
set -eu

REPO="Linuxero-dev/rustsysteminfo"
BIN_NAME="rsi"

usage() {
    cat <<EOF
Uso: install.sh [DIRECTORIO]
     install.sh --uninstall [DIRECTORIO]

DIRECTORIO   destino del binario (por defecto: \$HOME/.local/bin)

Variables de entorno:
  RSI_VERSION              instala una versión concreta (p. ej. v0.2.2) en vez de la última
  RSI_VERIFY_ATTESTATION=1 verifica además la procedencia con "gh attestation verify"
EOF
}

die() {
    echo "Error: $*" >&2
    exit 1
}

UNINSTALL=0
INSTALL_DIR=""
for arg in "$@"; do
    case "$arg" in
    --uninstall) UNINSTALL=1 ;;
    -h | --help)
        usage
        exit 0
        ;;
    -*)
        usage >&2
        die "opción desconocida: $arg"
        ;;
    *)
        [ -z "$INSTALL_DIR" ] || die "solo se admite un directorio de destino"
        INSTALL_DIR=$arg
        ;;
    esac
done

if [ -z "$INSTALL_DIR" ]; then
    [ -n "${HOME:-}" ] || die "HOME no está definido; indica un directorio de destino"
    INSTALL_DIR="$HOME/.local/bin"
fi

# Ruta absoluta: más abajo se cambia de directorio y una ruta relativa dejaría de valer.
case "$INSTALL_DIR" in
/*) ;;
*) INSTALL_DIR="$(pwd)/$INSTALL_DIR" ;;
esac

# La página de manual solo se gestiona en instalaciones del tipo <prefijo>/bin.
MAN_DIR=""
if [ "$(basename "$INSTALL_DIR")" = "bin" ]; then
    MAN_DIR="$(dirname "$INSTALL_DIR")/share/man/man1"
fi

if [ "$UNINSTALL" -eq 1 ]; then
    echo "Desinstalando $BIN_NAME de $INSTALL_DIR..."
    rm -f -- "$INSTALL_DIR/$BIN_NAME"
    if [ -n "$MAN_DIR" ]; then
        rm -f -- "$MAN_DIR/$BIN_NAME.1"
    fi
    echo "Desinstalado con éxito."
    exit 0
fi

command -v curl >/dev/null 2>&1 || die "se necesita curl"
command -v tar >/dev/null 2>&1 || die "se necesita tar"

ARCH=$(uname -m)
case "$ARCH" in
x86_64) TARGET="x86_64-unknown-linux-musl" ;;
aarch64 | arm64) TARGET="aarch64-unknown-linux-musl" ;;
*) die "arquitectura no soportada ($ARCH)." ;;
esac

ASSET_NAME="${BIN_NAME}-${TARGET}.tar.gz"
CHECKSUM_NAME="${ASSET_NAME}.sha256"

if [ -n "${RSI_VERSION:-}" ]; then
    case "$RSI_VERSION" in
    *[!A-Za-z0-9._-]*) die "RSI_VERSION contiene caracteres no válidos" ;;
    esac
    BASE_URL="https://github.com/$REPO/releases/download/$RSI_VERSION"
else
    BASE_URL="https://github.com/$REPO/releases/latest/download"
fi

sha_check() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "$1" >/dev/null 2>&1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -c "$1" >/dev/null 2>&1
    else
        die "se necesita sha256sum o shasum para verificar la descarga"
    fi
}

fetch() {
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 --max-time 120 "$1" -o "$2"
}

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM HUP

echo "Descargando $ASSET_NAME..."
fetch "$BASE_URL/$ASSET_NAME" "$TMP_DIR/$ASSET_NAME" || die "no se pudo descargar $ASSET_NAME"
fetch "$BASE_URL/$CHECKSUM_NAME" "$TMP_DIR/$CHECKSUM_NAME" || die "no se pudo descargar $CHECKSUM_NAME"

echo "Verificando integridad (SHA256)..."
(cd "$TMP_DIR" && sha_check "$CHECKSUM_NAME") || die "la suma de comprobación no coincide."

if [ "${RSI_VERIFY_ATTESTATION:-0}" = "1" ]; then
    command -v gh >/dev/null 2>&1 || die "RSI_VERIFY_ATTESTATION=1 requiere la CLI de GitHub (gh)"
    echo "Verificando procedencia (attestation)..."
    gh attestation verify "$TMP_DIR/$ASSET_NAME" --repo "$REPO" >/dev/null ||
        die "la verificación de procedencia ha fallado."
fi

# Se extrae en un directorio temporal y solo se instalan los ficheros esperados.
mkdir "$TMP_DIR/extract"
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR/extract" || die "el paquete descargado no es válido"
if [ ! -f "$TMP_DIR/extract/$BIN_NAME" ] || [ -L "$TMP_DIR/extract/$BIN_NAME" ]; then
    die "el paquete no contiene el binario $BIN_NAME"
fi

mkdir -p "$INSTALL_DIR"
echo "Instalando en $INSTALL_DIR/$BIN_NAME..."
# Se copia con otro nombre y se renombra: es atómico y funciona aunque rsi esté en ejecución.
install -m 755 "$TMP_DIR/extract/$BIN_NAME" "$INSTALL_DIR/.$BIN_NAME.new.$$"
mv -f "$INSTALL_DIR/.$BIN_NAME.new.$$" "$INSTALL_DIR/$BIN_NAME"

if [ -n "$MAN_DIR" ] && [ -f "$TMP_DIR/extract/$BIN_NAME.1" ]; then
    if mkdir -p "$MAN_DIR" && install -m 644 "$TMP_DIR/extract/$BIN_NAME.1" "$MAN_DIR/$BIN_NAME.1"; then
        echo "Página de manual instalada en $MAN_DIR/$BIN_NAME.1"
    else
        echo "Aviso: no se pudo instalar la página de manual." >&2
    fi
fi

case ":${PATH:-}:" in
*":$INSTALL_DIR:"*) ;;
*) echo "Aviso: $INSTALL_DIR no está en tu PATH." ;;
esac

echo "¡Instalación exitosa!"
echo "Ejecuta: $BIN_NAME"
