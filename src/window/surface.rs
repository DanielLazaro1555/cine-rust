// Construcción de la cabecera y la página inicial.
use super::*;

pub(super) fn build_headerbar() -> (adw::WindowTitle, adw::HeaderBar) {
        // Título de la barra superior
        let title_widget = adw::WindowTitle::new("Cine (Rust)", "");
        title_widget.set_visible(false);

        let headerbar = adw::HeaderBar::builder()
            .valign(gtk::Align::Start)
            .title_widget(&title_widget)
            .build();
        headerbar.add_css_class("osd");
        headerbar.add_css_class("header-bar");

        // Menú desplegable "Abrir" en la esquina superior izquierda (igual que el original)
        let open_menu = gio::Menu::new();
        let sec_open = gio::Menu::new();
        sec_open.append(Some("Abrir archivos…"), Some("win.clear-and-add"));
        sec_open.append(Some("Abrir carpeta…"), Some("win.open-folder"));
        sec_open.append(Some("Abrir URL…"), Some("win.open-url"));
        open_menu.append_section(None, &sec_open);

        let sec_add = gio::Menu::new();
        sec_add.append(Some("Añadir archivos a la lista…"), Some("win.add-playlist-files"));
        sec_add.append(Some("Añadir carpeta a la lista…"), Some("win.add-playlist-folder"));
        sec_add.append(Some("Añadir URL a la lista…"), Some("win.add-url"));
        open_menu.append_section(None, &sec_add);

        let sec_tracks = gio::Menu::new();
        sec_tracks.append(Some("Añadir pista de subtítulos…"), Some("win.add-sub-tracks"));
        sec_tracks.append(Some("Añadir pista de audio…"), Some("win.add-audio-tracks"));
        open_menu.append_section(None, &sec_tracks);

        let sec_hist = gio::Menu::new();
        sec_hist.append(Some("Historial de reproducción"), Some("win.open-history"));
        open_menu.append_section(None, &sec_hist);

        let open_menu_btn = gtk::MenuButton::builder()
            .label("Abrir")
            .always_show_arrow(true)
            .valign(gtk::Align::Center)
            .menu_model(&open_menu)
            .build();
        headerbar.pack_start(&open_menu_btn);

        // Menú principal en la esquina superior derecha
        let primary_menu = gio::Menu::new();
        let section_main = gio::Menu::new();
        section_main.append(Some("Nueva ventana"), Some("app.new-window"));
        section_main.append(Some("Preferencias"), Some("app.preferences"));
        section_main.append(Some("Atajos de teclado"), Some("win.custom-shortcuts"));
        section_main.append(Some("Acerca de Cine Rust"), Some("app.about"));
        section_main.append(Some("Salir"), Some("app.quit"));
        primary_menu.append_section(None, &section_main);

        let primary_menu_btn = gtk::MenuButton::builder()
            .valign(gtk::Align::Center)
            .primary(true)
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menú principal")
            .menu_model(&primary_menu)
            .build();
        headerbar.pack_end(&primary_menu_btn);

        (title_widget, headerbar)
}

pub(super) fn build_start_page() -> adw::StatusPage {
        // Página de inicio (AdwStatusPage) con todos los botones del diseño original
        let start_page = adw::StatusPage::builder()
            .margin_top(48)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .icon_name(ICON_NAME)
            .title("Cine (Rust)")
            .description("Arrastra y suelta archivos o carpetas aquí")
            .build();
        start_page.add_css_class("start-page");

        let start_buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        let top_open_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        top_open_row.add_css_class("open-btns-box");
        let open_btn = gtk::Button::builder()
            .label("Abrir…")
            .action_name("win.clear-and-add")
            .hexpand(true)
            .build();
        open_btn.add_css_class("suggested-action");
        open_btn.add_css_class("pill");

        let history_btn = gtk::Button::builder()
            .tooltip_text("Historial de reproducción")
            .icon_name("document-open-recent-symbolic")
            .action_name("win.open-history")
            .build();
        history_btn.add_css_class("suggested-action");
        history_btn.add_css_class("circular");
        history_btn.add_css_class("watch-history");

        top_open_row.append(&open_btn);
        top_open_row.append(&history_btn);
        start_buttons_box.append(&top_open_row);

        let open_folder_btn = gtk::Button::builder()
            .label("Abrir carpeta")
            .action_name("win.open-folder")
            .build();
        open_folder_btn.add_css_class("pill");
        start_buttons_box.append(&open_folder_btn);

        let open_url_btn = gtk::Button::builder()
            .label("Abrir URL")
            .action_name("win.open-url")
            .build();
        open_url_btn.add_css_class("pill");
        start_buttons_box.append(&open_url_btn);

        start_page.set_child(Some(&start_buttons_box));

        start_page
}
