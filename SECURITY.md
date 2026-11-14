# Política de seguridad

## Cómo informar de una vulnerabilidad

No abras un issue público. Usa el aviso privado de GitHub:
**Security → Report a vulnerability** en
<https://github.com/Linuxero-dev/rustsysteminfo/security/advisories/new>.

## Verificar las descargas

Cada release publica, junto a cada `.tar.gz`, su suma `.sha256` y una
*attestation* de procedencia generada por GitHub Actions:

```sh
gh attestation verify rsi-x86_64-unknown-linux-musl.tar.gz --repo Linuxero-dev/rustsysteminfo
```

El instalador comprueba el SHA256 siempre, y la procedencia si se ejecuta con
`RSI_VERIFY_ATTESTATION=1` (requiere la CLI `gh`).
