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

El instalador comprueba el SHA256 siempre. La procedencia se verifica además
con la CLI `gh` cuando está instalada (`RSI_VERIFY_ATTESTATION=auto`, el valor
por defecto): si no se puede verificar, avisa y continúa, porque el SHA256 ya
coincide. Con `RSI_VERIFY_ATTESTATION=1` la verificación es obligatoria y la
instalación falla si no se completa; con `0` se omite.

Los paquetes `.deb` se publican igual: cada uno con su `.sha256` y su *attestation*
(`gh attestation verify rustsysteminfo_<versión>_amd64.deb --repo Linuxero-dev/rustsysteminfo`).
El paquete no ejecuta nada al instalarse salvo un aviso informativo (`postinst`).
