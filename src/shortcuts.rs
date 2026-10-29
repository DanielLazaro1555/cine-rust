// shortcuts.rs
//
// Atajos de teclado internos y diálogo de atajos (Shortcuts) completamente en español.
// Equivalente modular a `shortcuts.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;

const SHORTCUTS_UI_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <requires lib="gtk" version="4.0"/>
  <object class="AdwShortcutsDialog" id="shortcuts_dialog">
    <property name="content-width">540</property>
    <child>
      <object class="AdwShortcutsSection">
        <property name="title">General</property>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Nueva ventana</property>
            <property name="action-name">app.new-window</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Abrir archivos…</property>
            <property name="action-name">win.clear-and-add</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Abrir carpeta…</property>
            <property name="action-name">win.open-folder</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Abrir URL…</property>
            <property name="action-name">win.open-url</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Añadir archivos a la lista…</property>
            <property name="action-name">win.add-playlist-files</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Añadir carpeta a la lista…</property>
            <property name="action-name">win.add-playlist-folder</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Añadir URL a la lista…</property>
            <property name="action-name">win.add-url</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Lista de reproducción</property>
            <property name="action-name">win.open-playlist-dialog</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Historial de reproducción</property>
            <property name="action-name">win.open-history</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Preferencias</property>
            <property name="action-name">app.preferences</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Mostrar atajos de teclado</property>
            <property name="action-name">win.custom-shortcuts</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Cerrar ventana</property>
            <property name="action-name">win.quit</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Salir de la aplicación</property>
            <property name="action-name">app.quit</property>
          </object>
        </child>
      </object>
    </child>
    <child>
      <object class="AdwShortcutsSection">
        <property name="title">Reproducción</property>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Reproducir / Pausar</property>
            <property name="accelerator">space</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Pantalla completa</property>
            <property name="accelerator">f</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Retroceder 5 segundos</property>
            <property name="accelerator">Left</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Avanzar 5 segundos</property>
            <property name="accelerator">Right</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Alternar pista de audio</property>
            <property name="accelerator">a</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Alternar subtítulos</property>
            <property name="accelerator">s</property>
          </object>
        </child>
        <child>
          <object class="AdwShortcutsItem">
            <property name="title">Información técnica y estadísticas</property>
            <property name="accelerator">i</property>
          </object>
        </child>
      </object>
    </child>
  </object>
</interface>
"#;

/// Abre el diálogo interactivo de atajos de teclado traducido completamente al español.
pub fn present_shortcuts_dialog(parent: &impl IsA<gtk::Widget>) {
    let builder = gtk::Builder::from_string(SHORTCUTS_UI_XML);
    if let Some(dialog) = builder.object::<adw::Dialog>("shortcuts_dialog") {
        dialog.present(Some(parent));
    }
}

/// Registra los aceleradores de teclado principales en la aplicación.
pub fn register_window_accels(app: &adw::Application) {
    app.set_accels_for_action("win.open-folder", &["<Primary>i"]);
    app.set_accels_for_action("win.open-url", &["<Primary>u"]);
    app.set_accels_for_action("win.add-url", &["<Shift><Primary>u"]);
    app.set_accels_for_action("win.open-history", &["<Primary>h"]);
    app.set_accels_for_action("win.add-playlist-folder", &["<Shift><Primary>i"]);
    app.set_accels_for_action("win.open-playlist-dialog", &["<Primary>p"]);
    app.set_accels_for_action("win.clear-and-add", &["<Primary>o"]);
    app.set_accels_for_action("win.add-playlist-files", &["<Shift><Primary>o"]);
    app.set_accels_for_action("win.custom-shortcuts", &["<Primary>question"]);
    app.set_accels_for_action("win.quit", &["q", "<Primary>w"]);
}
