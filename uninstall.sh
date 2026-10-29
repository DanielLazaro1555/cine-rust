#!/usr/bin/env bash
# uninstall.sh
#
# Desinstala Cine Rust de forma limpia del entorno de usuario (~/.local).
#
# SPDX-License-Identifier: GPL-3.0-or-later

set -e

rm -f "$HOME/.local/bin/cine"
rm -f "$HOME/.local/bin/cine-rust"
rm -f "$HOME/.local/share/applications/io.github.diegopvlk.Cine.desktop"
rm -f "$HOME/.local/share/applications/io.github.daniellazaro.CineRust.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/io.github.diegopvlk.Cine.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

echo "Cine fue desinstalado y limpiado de ~/.local con éxito."
