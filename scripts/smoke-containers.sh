#!/usr/bin/env bash
# Instala los paquetes de un directorio en contenedores de varias distribuciones y
# comprueba que rsi funciona. Necesita docker.
#
# Uso: scripts/smoke-containers.sh DIRECTORIO [deb|rpm|all]
set -uo pipefail

DIR="${1:-}"
KIND="${2:-all}"
[ -d "$DIR" ] || { echo "Uso: $0 DIRECTORIO [deb|rpm|all]" >&2; exit 2; }
case "$KIND" in deb | rpm | all) ;; *) echo "Tipo no válido: $KIND" >&2; exit 2 ;; esac
command -v docker >/dev/null 2>&1 || { echo "Se necesita docker." >&2; exit 1; }
DIR="$(cd "$DIR" && pwd)"

DEB_IMAGES=(ubuntu:22.04 ubuntu:24.04 debian:12)

fails=0

# Prueba común: el binario instalado debe responder a --version; --once es solo informativa.
CHECK='rsi --version && { rsi --once >/dev/null 2>&1 || echo "AVISO: rsi --once devolvió error en este contenedor"; }'

smoke() {
    local image="$1" install="$2"
    echo "==> $image"
    if docker run --rm -v "$DIR:/pkg:ro" "$image" sh -c "$install && $CHECK"; then
        echo "    ok"
    else
        echo "    FALLO en $image"
        fails=$((fails + 1))
    fi
}

if [ "$KIND" = deb ] || [ "$KIND" = all ]; then
    for image in "${DEB_IMAGES[@]}"; do
        smoke "$image" "apt-get install -y /pkg/*_\$(dpkg --print-architecture).deb >/dev/null"
    done
fi

if [ "$KIND" = rpm ] || [ "$KIND" = all ]; then
    smoke fedora:latest "dnf install -y /pkg/*.\$(uname -m).rpm >/dev/null"
    smoke opensuse/tumbleweed "rpm -i --nosignature /pkg/*.\$(uname -m).rpm"
fi

if [ "$fails" -ne 0 ]; then
    echo "$fails distribución(es) con fallos"
    exit 1
fi
echo "Todas las distribuciones probadas funcionan."
