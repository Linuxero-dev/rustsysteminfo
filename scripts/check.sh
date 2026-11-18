#!/usr/bin/env bash
# Comprobación previa a subir cambios: ejecútala antes de cada push.
# Para que se ejecute sola:  ln -sf ../../scripts/check.sh .git/hooks/pre-push
set -uo pipefail
cd "$(dirname "$(readlink -f "$0")")/.." || exit 1

fails=0
warns=0

step() {
    local desc="$1"
    shift
    printf '==> %s\n' "$desc"
    if "$@"; then
        printf '    ok\n'
    else
        printf '    FALLO\n'
        fails=$((fails + 1))
    fi
}

warn() {
    printf '    AVISO: %s\n' "$*"
    warns=$((warns + 1))
}

have() { command -v "$1" >/dev/null 2>&1; }


step "Estamos dentro de un repositorio git" git rev-parse --is-inside-work-tree >/dev/null

origin="$(git remote get-url origin 2>/dev/null || true)"
if [ -z "$origin" ]; then
    warn "no hay remoto 'origin'"
elif ! printf '%s' "$origin" | grep -qi 'Linuxero-dev/rustsysteminfo'; then
    warn "origin apunta a '$origin', no a Linuxero-dev/rustsysteminfo"
fi

branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')"

step "Cargo.lock existe y no está ignorado por git" \
    bash -c 'test -f Cargo.lock && ! git check-ignore -q Cargo.lock'
step "No hay artefactos de compilación versionados (target/)" \
    bash -c '! git ls-files | grep -q "^target/"'
step "No hay secretos evidentes en el código" \
    bash -c '! git grep -nIE --untracked "(ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[0-9A-Z]{16}|-----BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY-----)" -- . ":(exclude)scripts/check.sh"'

step "Formato (cargo fmt)" cargo fmt --all --check
step "Lints (cargo clippy, avisos = error)" cargo clippy --all-targets --locked -- -D warnings
step "Tests (cargo test)" cargo test --locked

if have script; then
    step "Tests con terminal simulada" script -qec "cargo test --locked" /dev/null
else
    warn "'script' no está instalado: se omiten los tests con terminal simulada"
fi
step "Compilación release" cargo build --release --locked
step "Ejecución: rsi --help" ./target/release/rsi --help >/dev/null
step "Ejecución: rsi --once" ./target/release/rsi --once >/dev/null
step "Con la salida redirigida no se emiten secuencias de escape" \
    bash -c '! ./target/release/rsi --once | grep -q $'"'"'\x1b'"'"

if have dpkg-deb; then
    step "Pruebas del paquete .deb" ./scripts/test-deb.sh
else
    warn "dpkg-deb no está instalado: se omiten las pruebas del .deb"
fi

if have rpmbuild && have rpm; then
    step "Pruebas del paquete .rpm" ./scripts/test-rpm.sh
else
    warn "rpm/rpmbuild no están instalados: se omiten las pruebas del .rpm (sudo apt install rpm)"
fi

step "Sintaxis de install.sh" sh -n install.sh
if have shellcheck; then
    step "shellcheck" shellcheck install.sh scripts/*.sh
else
    warn "shellcheck no está instalado (sudo apt install shellcheck)"
fi
step "Pruebas del instalador" ./scripts/test-install.sh

if have cargo-audit; then
    step "Vulnerabilidades conocidas (cargo audit)" cargo audit
else
    warn "cargo-audit no está instalado (cargo install cargo-audit --locked)"
fi

echo
echo "Versión en Cargo.toml: $(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
echo "Rama: $branch"
echo "Cambios pendientes:"
git status --short | sed 's/^/    /'
echo
if [ "$fails" -ne 0 ]; then
    echo "NO subas todavía: $fails comprobación(es) fallida(s), $warns aviso(s)."
    exit 1
fi
echo "Todo correcto ($warns aviso(s)). Puedes hacer push."
