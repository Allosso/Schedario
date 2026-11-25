#!/bin/sh
# Compila la versione definitiva per Linux o macOS e la copia in dist/.
# Uso (nella cartella Note):  sh script/compila-linux-macos.sh
#
# Requisiti: Rust installato con rustup (https://rustup.rs).
# Linux, per eseguire: librerie OpenGL/EGL, libxkbcommon, X11 o Wayland
# (gia' presenti in qualsiasi desktop moderno) e xdg-desktop-portal per le
# finestre di scelta delle cartelle.
set -e
cd "$(dirname "$0")/.."

cargo test
cargo build --release

mkdir -p dist
cp target/release/schedario dist/schedario
cp LICENSE dist/LICENSE.txt
cp docs/LICENZE-TERZE-PARTI.md dist/
# Testi completi delle licenze (generati da script/genera-licenze.ps1).
cp assets/licenze/licenze.txt dist/LICENZE.txt

# Versione: la stessa di Cargo.toml.
VERSIONE=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

if [ "$(uname)" = "Darwin" ]; then
    # Pacchetto .app minimo, avviabile con doppio clic dal Finder.
    APP="dist/Schedario.app/Contents"
    mkdir -p "$APP/MacOS"
    cp target/release/schedario "$APP/MacOS/schedario"
    cat > "$APP/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Schedario</string>
  <key>CFBundleIdentifier</key><string>io.github.allosso.schedario</string>
  <key>CFBundleExecutable</key><string>schedario</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSIONE</string>
  <key>CFBundleVersion</key><string>$VERSIONE</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSLocalNetworkUsageDescription</key><string>Sincronizza le note con gli altri dispositivi della rete locale.</string>
</dict></plist>
PLIST
    echo "Pronto: dist/Schedario.app"
else
    echo "Pronto: dist/schedario"
fi
