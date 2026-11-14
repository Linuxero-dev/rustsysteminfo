#!/bin/sh
set -e

REPO="Linuxero-dev/rustsysteminfo"
BIN_NAME="rsi"
INSTALL_DIR="${1:-$HOME/.local/bin}"

if [ "$1" = "--uninstall" ]; then
    echo "Desinstalando $BIN_NAME..."
    rm -f "$INSTALL_DIR/$BIN_NAME"
    echo "Desinstalado con éxito."
    exit 0
fi

ARCH=$(uname -m)
case "$ARCH" in
    x86_64) TARGET="x86_64-unknown-linux-musl" ;;
    aarch64|arm64) TARGET="aarch64-unknown-linux-musl" ;;
    *) echo "Error: Arquitectura no soportada ($ARCH)."; exit 1 ;;
esac

echo "Buscando la última versión de $REPO..."
API_URL="https://api.github.com/repos/$REPO/releases/latest"
TAG=$(curl -s "$API_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
    echo "Error: No se pudo obtener la última versión."
    exit 1
fi

ASSET_NAME="${BIN_NAME}-${TARGET}.tar.gz"
CHECKSUM_NAME="${ASSET_NAME}.sha256"
BASE_URL="https://github.com/$REPO/releases/download/$TAG"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Descargando $ASSET_NAME ($TAG)..."
curl -sL "$BASE_URL/$ASSET_NAME" -o "$TMP_DIR/$ASSET_NAME"
curl -sL "$BASE_URL/$CHECKSUM_NAME" -o "$TMP_DIR/$CHECKSUM_NAME"

echo "Verificando integridad (SHA256)..."
cd "$TMP_DIR"
if ! sha256sum -c "$CHECKSUM_NAME" > /dev/null 2>&1; then
    echo "Error: La suma de comprobación no coincide."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
echo "Instalando en $INSTALL_DIR/$BIN_NAME..."
tar -xzf "$ASSET_NAME" -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "¡Instalación exitosa!"
echo "Ejecuta: $BIN_NAME"
