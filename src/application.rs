// application.rs
//
// Clase de aplicación principal `CineApplication`: ciclo de vida, acciones
// globales (`app.new-window`, `app.preferences`, `app.about`, `app.quit`) y apertura de archivos.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::mpris::MprisController;
use crate::preferences::{AppSettings, SharedSettings, present_preferences_dialog};
use crate::shortcuts::register_window_accels;
use crate::window::CineWindow;
use crate::{APP_ID, ICON_NAME, VERSION};

pub struct CineApplication {
    app: adw::Application,
}

impl CineApplication {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::HANDLES_OPEN)
            .resource_base_path("/io/github/diegopvlk/Cine")
            .build();

        app.add_main_option(
            "new-window",
            glib::Char::from(b'n'),
            glib::OptionFlags::NONE,
            glib::OptionArg::None,
            "Abrir una nueva ventana de Cine",
            None,
        );

        let settings: SharedSettings = Rc::new(RefCell::new(AppSettings::load_from_disk()));
        let mpris = MprisController::new();

        {
            let settings = settings.clone();
            app.connect_startup(move |app| {
                Self::on_startup(app, settings.clone());
            });
        }

        {
            let settings = settings.clone();
            let mpris = mpris.clone();
            app.connect_activate(move |app| {
                let win = CineWindow::new(app, settings.clone(), mpris.clone());
                win.present();
            });
        }

        {
            let settings = settings.clone();
            let mpris = mpris.clone();
            app.connect_open(move |app, files, _hint| {
                let win = CineWindow::new(app, settings.clone(), mpris.clone());
                win.present();
                win.open_files(files, true);
            });
        }

        Self { app }
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }

    fn on_startup(app: &adw::Application, settings: SharedSettings) {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

        // Acción: Nueva ventana (Ctrl+N)
        let action_new_win = gio::SimpleAction::new("new-window", None);
        let app_weak = app.downgrade();
        action_new_win.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.activate();
            }
        });
        app.add_action(&action_new_win);
        app.set_accels_for_action("app.new-window", &["<Primary>n"]);

        // Acción: Preferencias (Ctrl+,)
        let action_prefs = gio::SimpleAction::new("preferences", None);
        let app_weak = app.downgrade();
        action_prefs.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(win) = app.active_window() {
                    present_preferences_dialog(&win, settings.clone());
                }
            }
        });
        app.add_action(&action_prefs);
        app.set_accels_for_action("app.preferences", &["<Primary>comma"]);

        // Acción: Salir (Ctrl+Q)
        let action_quit = gio::SimpleAction::new("quit", None);
        let app_weak = app.downgrade();
        action_quit.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                for win in app.windows() {
                    win.close();
                }
                app.quit();
            }
        });
        app.add_action(&action_quit);
        app.set_accels_for_action("app.quit", &["<Primary>q"]);

        // Acción: Acerca de Cine Rust
        let action_about = gio::SimpleAction::new("about", None);
        let app_weak = app.downgrade();
        action_about.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                Self::show_about_dialog(&app);
            }
        });
        app.add_action(&action_about);

        register_window_accels(app);
    }

    fn show_about_dialog(app: &adw::Application) {
        let about = adw::AboutDialog::builder()
            .application_name("Cine (Rust)")
            .application_icon(ICON_NAME)
            .developer_name("Daniel Lazaro (basado en Cine de Diego Povliuk)")
            .version(VERSION)
            .copyright("© 2026 Daniel Lazaro / Diseño original © Diego Povliuk")
            .issue_url("https://github.com/DanielLazaro1555/cine-rust/issues")
            .license_type(gtk::License::Gpl30)
            .build();

        about.add_acknowledgement_section(
            None,
            &[
                "MPV https://mpv.io/",
                "Cine (Original) https://github.com/diegopvlk/Cine",
                "gtk4-rs https://gtk-rs.org/",
            ],
        );

        if let Some(win) = app.active_window() {
            about.present(Some(&win));
        } else {
            about.present(None::<&gtk::Widget>);
        }
    }
}
