# Especificación del paquete .rpm de rsi.
# No compila nada: empaqueta ficheros ya preparados por scripts/make-rpm.sh.
# Se invoca con:  --define "rsi_version X.Y.Z" --define "rsi_stage /ruta/al/arbol"

# Binario estático ya compilado y sin símbolos: nada que extraer, quitar ni recomprimir
# (el strip y los scripts de rpmbuild del equipo no entienden otras arquitecturas).
%global debug_package %{nil}
%global __strip /bin/true
%global __os_install_post %{nil}
AutoReqProv: no

Name:           rustsysteminfo
Version:        %{rsi_version}
Release:        1
Summary:        Información del sistema con logo animado y monitor en vivo
License:        MIT
URL:            https://github.com/Linuxero-dev/rustsysteminfo

%description
rsi es una alternativa moderna a Neofetch y Fastfetch: muestra los datos del
sistema junto a un logo animado y mide CPU, RAM y swap en vivo.

Es un binario estático de Rust, sin dependencias en tiempo de ejecución.

%install
mkdir -p %{buildroot}
cp -a %{rsi_stage}/. %{buildroot}/

%post
if [ -x /usr/local/bin/rsi ]; then
    echo "Aviso: existe /usr/local/bin/rsi (instalado con el script) y tiene prioridad sobre /usr/bin/rsi." >&2
    echo "       Para usar la del paquete: sudo rm /usr/local/bin/rsi" >&2
fi
exit 0

%files
%defattr(-,root,root,-)
%{_bindir}/rsi
%{_mandir}/man1/rsi.1*
%{_datadir}/bash-completion/completions/rsi
%{_datadir}/zsh/site-functions/_rsi
%{_datadir}/fish/vendor_completions.d/rsi.fish
%license /usr/share/licenses/rustsysteminfo/LICENSE

%changelog
* Tue Nov 17 2026 Linuxero-dev - %{version}-1
- Paquete inicial para Fedora, openSUSE y derivadas.
