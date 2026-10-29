#!/usr/bin/env bash
# install.sh
#
# Instala Cine Rust de forma limpia en el entorno de usuario (~/.local).
# Elimina versiones anteriores obsoletas o duplicadas y registra el lanzador
# y los tipos MIME para que aparezca en el menú y en "Abrir con..." al dar clic derecho.
#
# SPDX-License-Identifier: GPL-3.0-or-later

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> [1/5] Compilando binario optimizado en modo release..."
cargo build --release

BIN_DEST="$HOME/.local/bin"
DESKTOP_DEST="$HOME/.local/share/applications"
ICON_DEST="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$BIN_DEST"
mkdir -p "$DESKTOP_DEST"
mkdir -p "$ICON_DEST"

echo "==> [2/5] Limpiando enlaces o versiones duplicadas anteriores..."
rm -f "$BIN_DEST/cine-rust"
rm -f "$BIN_DEST/cine"
rm -f "$DESKTOP_DEST/io.github.daniellazaro.CineRust.desktop"

echo "==> [3/5] Instalando ejecutable en $BIN_DEST/cine..."
cp "$SCRIPT_DIR/target/release/cine" "$BIN_DEST/cine"
chmod +x "$BIN_DEST/cine"

echo "==> [4/5] Instalando icono oficial..."
if [ -f "$SCRIPT_DIR/data/icons/scalable/apps/io.github.diegopvlk.Cine.svg" ]; then
    cp "$SCRIPT_DIR/data/icons/scalable/apps/io.github.diegopvlk.Cine.svg" "$ICON_DEST/io.github.diegopvlk.Cine.svg"
fi

echo "==> [5/5] Instalando lanzador oficial y registrando tipos MIME..."
cat > "$DESKTOP_DEST/io.github.diegopvlk.Cine.desktop" << EOF
[Desktop Entry]
Name=Cine (Rust)
GenericName=Reproductor de Video
Comment=Reproduce tus videos favoritos
Exec=$BIN_DEST/cine %U
Icon=io.github.diegopvlk.Cine
Terminal=false
Type=Application
Categories=AudioVideo;Video;Player;GTK;GNOME;
Keywords=Audio;Video;Movie;Película;TV;Film;Clip;Player;Cine;mpv;Rust;
MimeType=video/mp4;video/x-matroska;video/webm;video/quicktime;video/x-msvideo;video/avi;video/x-flv;video/x-m4v;video/mpeg;video/ogg;video/3gp;video/x-theora;video/mp2t;audio/mp3;audio/flac;audio/ogg;audio/opus;audio/wav;audio/x-matroska;audio/m4a;audio/aac;inode/directory;
StartupNotify=true
Actions=new-window;

[Desktop Action new-window]
Name=Nueva ventana
Exec=$BIN_DEST/cine --new-window
EOF

chmod +x "$DESKTOP_DEST/io.github.diegopvlk.Cine.desktop"

# Actualizar base de datos de escritorio y caché de iconos
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DEST" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo "=========================================================="
echo " ¡Cine (Rust) se instaló y limpió con éxito!"
echo " - Ejecutable: $BIN_DEST/cine"
echo " - Lanzador:   $DESKTOP_DEST/io.github.diegopvlk.Cine.desktop"
echo " Ya figura como 'Cine (Rust)' en 'Abrir con...' y en el menú del sistema."
echo "=========================================================="
