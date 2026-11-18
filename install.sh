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

warn() {
    echo "Aviso: $*" >&2
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
set_man_dir() {
    MAN_DIR=""
    if [ "$(basename "$INSTALL_DIR")" = "bin" ]; then
        MAN_DIR="$(dirname "$INSTALL_DIR")/share/man/man1"
    fi
}
set_man_dir

is_system_dir() {
    [ "$INSTALL_DIR" = "/usr/local/bin" ] || [ "$INSTALL_DIR" = "/usr/bin" ]
}

# ¿Se puede usar sudo? Si hace falta contraseña la pide en la terminal; falla si no hay
# permisos o no hay terminal (contenedores, CI).
sudo_works() {
    command -v sudo >/dev/null 2>&1 && sudo -v 2>/dev/null
}

# Ejecuta con sudo solo si NEED_SUDO=1.
NEED_SUDO=0
priv() {
    if [ "$NEED_SUDO" -eq 1 ]; then
        sudo "$@"
    else
        "$@"
    fi
}

if [ "$UNINSTALL" -eq 1 ]; then
    echo "Desinstalando $BIN_NAME de $INSTALL_DIR..."
    if is_system_dir && [ ! -w "$INSTALL_DIR" ]; then
        sudo_works || die "no se pudo desinstalar: necesitas permisos de administrador (usa sudo)"
        NEED_SUDO=1
    fi
    priv rm -f -- "$INSTALL_DIR/$BIN_NAME" || die "no se pudo borrar $INSTALL_DIR/$BIN_NAME (permisos)"
    if [ -n "$MAN_DIR" ]; then
        priv rm -f -- "$MAN_DIR/$BIN_NAME.1" || warn "no se pudo borrar la página de manual"
    fi
    if [ -n "${XDG_CACHE_HOME:-}" ] || [ -n "${HOME:-}" ]; then
        CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/$BIN_NAME"
        case "$CACHE_DIR" in
            /?*/"$BIN_NAME") rm -rf -- "$CACHE_DIR" 2>/dev/null || true ;;
        esac
    fi
    echo "Desinstalado con éxito."
    exit 0
fi

ATTEST="${RSI_VERIFY_ATTESTATION:-auto}"
case "$ATTEST" in
    0 | 1 | auto) ;;
    *) die "RSI_VERIFY_ATTESTATION debe ser auto, 0 o 1" ;;
esac

# Descargador: curl, o wget si no hay curl (Alpine y Debian mínimos suelen venir sin curl).
if command -v curl >/dev/null 2>&1; then
    DOWNLOADER=curl
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER=wget
else
    die "se necesita curl o wget (p. ej.: sudo apt install curl | sudo dnf install curl | sudo apk add curl)"
fi
command -v tar >/dev/null 2>&1 || die "se necesita tar (p. ej.: sudo apt install tar | sudo dnf install tar | sudo apk add tar)"

ARCH=$(uname -m)
case "$ARCH" in
    x86_64 | amd64) TARGET="x86_64-unknown-linux-musl" ;;
    aarch64 | arm64) TARGET="aarch64-unknown-linux-musl" ;;
    *) die "arquitectura no soportada ($ARCH). Hay binarios para x86_64 y aarch64; en otras, compila con: cargo install --git https://github.com/$REPO" ;;
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
        die "se necesita sha256sum o shasum para verificar la descarga (paquete coreutils)"
    fi
}

fetch() {
    if [ "$DOWNLOADER" = curl ]; then
        curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 --max-time 120 "$1" -o "$2"
    else
        wget -q -O "$2" "$1"
    fi
}

TMP_DIR=$(mktemp -d) || die "no se pudo crear un directorio temporal"
trap 'rm -rf "$TMP_DIR"' EXIT
trap 'exit 1' INT TERM HUP

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
                warn "no se pudo verificar la procedencia (¿gh sin autenticar?); el SHA256 sí coincide."
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

if is_system_dir && [ ! -w "$INSTALL_DIR" ]; then
    if sudo_works; then
        NEED_SUDO=1
        echo "Se requiere sudo para instalar en $INSTALL_DIR"
    else
        [ -n "${HOME:-}" ] || die "no hay permisos en $INSTALL_DIR, sudo no está disponible y HOME no está definido; indica un directorio de destino"
        warn "no se puede escribir en $INSTALL_DIR (sin permisos o sin sudo utilizable)."
        echo "       Instalando en \$HOME/.local/bin en su lugar." >&2
        INSTALL_DIR="$HOME/.local/bin"
        set_man_dir
    fi
fi

# Si ya hay un paquete .deb de rsi, la copia nueva en /usr/local/bin lo tapará.
if [ "$INSTALL_DIR" = "/usr/local/bin" ] && command -v dpkg >/dev/null 2>&1 && dpkg -s rustsysteminfo >/dev/null 2>&1; then
    warn "ya hay un paquete .deb de rsi instalado (rustsysteminfo, en /usr/bin)."
    echo "       Esta instalación, en /usr/local/bin, tendrá prioridad sobre él." >&2
    echo "       Para quedarte con una sola copia: sudo apt remove rustsysteminfo" >&2
fi

priv mkdir -p "$INSTALL_DIR" || die "no se pudo crear $INSTALL_DIR"

echo "Instalando en $INSTALL_DIR/$BIN_NAME..."
TMP_BIN="$INSTALL_DIR/.$BIN_NAME.new.$$"
if ! priv install -m 755 "$TMP_DIR/extract/$BIN_NAME" "$TMP_BIN" ||
    ! priv mv -f "$TMP_BIN" "$INSTALL_DIR/$BIN_NAME"; then
    priv rm -f -- "$TMP_BIN" 2>/dev/null || true
    die "no se pudo instalar en $INSTALL_DIR"
fi

if [ -n "$MAN_DIR" ] && [ -f "$TMP_DIR/extract/$BIN_NAME.1" ]; then
    if priv mkdir -p "$MAN_DIR" && priv install -m 644 "$TMP_DIR/extract/$BIN_NAME.1" "$MAN_DIR/$BIN_NAME.1"; then
        echo "Página de manual instalada en $MAN_DIR/$BIN_NAME.1"
    else
        warn "no se pudo instalar la página de manual."
    fi
fi

# Comprobación real: el binario se ejecuta (detecta noexec, arquitectura equivocada, etc.).
if ! VERSION_OUT=$("$INSTALL_DIR/$BIN_NAME" --version 2>&1); then
    echo "Error: se instaló $INSTALL_DIR/$BIN_NAME pero no se puede ejecutar:" >&2
    echo "       $VERSION_OUT" >&2
    echo "       Comprueba que el destino no está montado con noexec y que la arquitectura ($ARCH) es la correcta." >&2
    exit 1
fi

echo "¡Instalación exitosa! ($VERSION_OUT)"

# ¿Qué ejecutará el usuario al escribir «rsi»?
FOUND=$(command -v "$BIN_NAME" 2>/dev/null || true)
if [ "$FOUND" = "$INSTALL_DIR/$BIN_NAME" ]; then
    echo "Ejecuta: $BIN_NAME"
elif [ -n "$FOUND" ]; then
    warn "al escribir '$BIN_NAME' se ejecutará otro programa: $FOUND"
    echo "       Para usar el recién instalado: $INSTALL_DIR/$BIN_NAME" >&2
    echo "       (o pon $INSTALL_DIR antes en tu PATH, o borra el otro)." >&2
else
    echo ""
    case "$(basename "${SHELL:-sh}")" in
        fish)
            echo "'$BIN_NAME' aún no está en tu PATH. Para añadirlo de forma permanente:"
            echo "  fish_add_path \"$INSTALL_DIR\""
            ;;
        zsh)
            echo "'$BIN_NAME' aún no está en tu PATH. Para añadirlo de forma permanente:"
            echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
            ;;
        *)
            echo "'$BIN_NAME' aún no está en tu PATH. Para añadirlo de forma permanente:"
            echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
            ;;
    esac
    echo ""
    echo "Abre una terminal nueva, o ejecuta ahora con: $INSTALL_DIR/$BIN_NAME"
    echo "(Si el sistema sugiere 'sudo apt install rsi', ignóralo: es otro paquete.)"
fi
