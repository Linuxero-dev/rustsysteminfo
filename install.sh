#!/bin/sh
# Instalador de rsi (rustsysteminfo).
# Descarga el binario del último release de GitHub, verifica su SHA256 y lo instala.
set -eu

REPO="Linuxero-dev/rustsysteminfo"
BIN_NAME="rsi"

usage() {
    cat <<EOF
Uso: install.sh [OPCIONES] [DIRECTORIO]

OPCIONES:
  --local       instala en \$HOME/.local/bin sin sudo (por defecto si no hay sudo)
  --uninstall   desinstala rsi
  -h, --help    muestra esta ayuda

DIRECTORIO   destino del binario (por defecto: /usr/local/bin o \$HOME/.local/bin con --local)

Variables de entorno:
  RSI_VERSION              instala una versión concreta (p. ej. v0.2.2) en vez de la última
  RSI_VERIFY_ATTESTATION   verificación de procedencia con "gh attestation verify":
                           auto (por defecto) la comprueba si gh está instalado y avisa si falla
                           1  la exige: la instalación falla si no se puede verificar
                           0  la omite
EOF
}

die() {
    echo "Error: $*" >&2
    exit 1
}

UNINSTALL=0
LOCAL_INSTALL=0
INSTALL_DIR=""

for arg in "$@"; do
    case "$arg" in
        --uninstall) UNINSTALL=1 ;;
        --local) LOCAL_INSTALL=1 ;;
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
    if [ "$LOCAL_INSTALL" -eq 1 ]; then
        [ -n "${HOME:-}" ] || die "HOME no está definido; indica un directorio de destino"
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="/usr/local/bin"
    fi
fi

case "$INSTALL_DIR" in
    /*) ;;
    *) INSTALL_DIR="$(pwd)/$INSTALL_DIR" ;;
esac

MAN_DIR=""
if [ "$(basename "$INSTALL_DIR")" = "bin" ]; then
    MAN_DIR="$(dirname "$INSTALL_DIR")/share/man/man1"
fi

if [ "$UNINSTALL" -eq 1 ]; then
    echo "Desinstalando $BIN_NAME de $INSTALL_DIR..."
    if [ "$INSTALL_DIR" = "/usr/local/bin" ] || [ "$INSTALL_DIR" = "/usr/bin" ]; then
        if command -v sudo >/dev/null 2>&1; then
            sudo rm -f -- "$INSTALL_DIR/$BIN_NAME"
            [ -n "$MAN_DIR" ] && sudo rm -f -- "$MAN_DIR/$BIN_NAME.1"
        else
            rm -f -- "$INSTALL_DIR/$BIN_NAME" 2>/dev/null || die "no se pudo desinstalar (necesitas sudo)"
            [ -n "$MAN_DIR" ] && rm -f -- "$MAN_DIR/$BIN_NAME.1" 2>/dev/null || true
        fi
    else
        rm -f -- "$INSTALL_DIR/$BIN_NAME"
        [ -n "$MAN_DIR" ] && rm -f -- "$MAN_DIR/$BIN_NAME.1"
    fi
    echo "Desinstalado con éxito."
    exit 0
fi

ATTEST="${RSI_VERIFY_ATTESTATION:-auto}"
case "$ATTEST" in
    0 | 1 | auto) ;;
    *) die "RSI_VERIFY_ATTESTATION debe ser auto, 0 o 1" ;;
esac

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

verify_attestation() {
    gh attestation verify "$TMP_DIR/$ASSET_NAME" --repo "$REPO" >/dev/null
}

case "$ATTEST" in
    1)
        command -v gh >/dev/null 2>&1 || die "RSI_VERIFY_ATTESTATION=1 requiere la CLI de GitHub (gh)"
        echo "Verificando procedencia (attestation)..."
        verify_attestation || die "la verificación de procedencia ha fallado."
        ;;
    auto)
        if command -v gh >/dev/null 2>&1; then
            echo "Verificando procedencia (attestation)..."
            if ! verify_attestation 2>/dev/null; then
                echo "Aviso: no se pudo verificar la procedencia (¿gh sin autenticar?); el SHA256 sí coincide." >&2
                echo "       Usa RSI_VERIFY_ATTESTATION=1 para exigirla." >&2
            fi
        fi
        ;;
esac

mkdir "$TMP_DIR/extract"
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR/extract" || die "el paquete descargado no es válido"

if [ ! -f "$TMP_DIR/extract/$BIN_NAME" ] || [ -L "$TMP_DIR/extract/$BIN_NAME" ]; then
    die "el paquete no contiene el binario $BIN_NAME"
fi

NEED_SUDO=0
if [ "$INSTALL_DIR" = "/usr/local/bin" ] || [ "$INSTALL_DIR" = "/usr/bin" ]; then
    if [ ! -w "$INSTALL_DIR" ]; then
        if command -v sudo >/dev/null 2>&1; then
            NEED_SUDO=1
            echo "Se requiere sudo para instalar en $INSTALL_DIR"
        else
            echo "Aviso: no hay permisos para escribir en $INSTALL_DIR y sudo no está disponible." >&2
            echo "       Instalando en \$HOME/.local/bin en su lugar." >&2
            INSTALL_DIR="$HOME/.local/bin"
            if [ "$(basename "$INSTALL_DIR")" = "bin" ]; then
                MAN_DIR="$(dirname "$INSTALL_DIR")/share/man/man1"
            fi
            NEED_SUDO=0
        fi
    fi
fi

mkdir -p "$INSTALL_DIR" 2>/dev/null || {
    if [ "$NEED_SUDO" -eq 1 ]; then
        sudo mkdir -p "$INSTALL_DIR"
    else
        die "no se pudo crear $INSTALL_DIR"
    fi
}

echo "Instalando en $INSTALL_DIR/$BIN_NAME..."

if [ "$NEED_SUDO" -eq 1 ]; then
    sudo install -m 755 "$TMP_DIR/extract/$BIN_NAME" "$INSTALL_DIR/.$BIN_NAME.new.$$"
    sudo mv -f "$INSTALL_DIR/.$BIN_NAME.new.$$" "$INSTALL_DIR/$BIN_NAME"
else
    install -m 755 "$TMP_DIR/extract/$BIN_NAME" "$INSTALL_DIR/.$BIN_NAME.new.$$"
    mv -f "$INSTALL_DIR/.$BIN_NAME.new.$$" "$INSTALL_DIR/$BIN_NAME"
fi

if [ -n "$MAN_DIR" ] && [ -f "$TMP_DIR/extract/$BIN_NAME.1" ]; then
    if [ "$NEED_SUDO" -eq 1 ]; then
        if sudo mkdir -p "$MAN_DIR" && sudo install -m 644 "$TMP_DIR/extract/$BIN_NAME.1" "$MAN_DIR/$BIN_NAME.1"; then
            echo "Página de manual instalada en $MAN_DIR/$BIN_NAME.1"
        else
            echo "Aviso: no se pudo instalar la página de manual." >&2
        fi
    else
        if mkdir -p "$MAN_DIR" && install -m 644 "$TMP_DIR/extract/$BIN_NAME.1" "$MAN_DIR/$BIN_NAME.1"; then
            echo "Página de manual instalada en $MAN_DIR/$BIN_NAME.1"
        else
            echo "Aviso: no se pudo instalar la página de manual." >&2
        fi
    fi
fi

echo "¡Instalación exitosa!"

if [ "$INSTALL_DIR" != "/usr/local/bin" ] && [ "$INSTALL_DIR" != "/usr/bin" ]; then
    case ":${PATH:-}:" in
        *":$INSTALL_DIR:"*)
            echo "Ejecuta: $BIN_NAME"
            ;;
        *)
            echo ""
            echo "Para usar '$BIN_NAME' directamente, cierra y abre la terminal"
            echo "o ejecuta: export PATH=\"$INSTALL_DIR:\$PATH\""
            echo ""
            echo "Para hacerlo permanente: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo ""
            echo "Ejecuta ahora con: $INSTALL_DIR/$BIN_NAME"
            ;;
    esac
else
    echo "Ejecuta: $BIN_NAME"
fi
