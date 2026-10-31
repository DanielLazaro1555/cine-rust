// tracks.rs
//
// Selector y gestor interactivo de pistas de audio y subtítulos.
// Permite listar dinámicamente las pistas disponibles desde libmpv,
// activar pistas específicas con un clic y añadir pistas externas.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;

use crate::mpv_gl_area::VideoGLArea;

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub id: i64,
    pub track_type: String, // "audio" o "sub"
    pub title: String,
    pub lang: String,
    pub selected: bool,
    pub external: bool,
}

/// Traduce códigos de idioma ISO 639 a nombres legibles en español.
pub fn lang_code_to_name(code: &str) -> String {
    let lower = code.trim().to_lowercase();
    if lower.starts_with("es-us") || lower.starts_with("es-419") {
        "Español (Latinoamérica)".to_string()
    } else if lower.starts_with("es-es") {
        "Español (España)".to_string()
    } else if lower.starts_with("es") || lower.starts_with("spa") {
        "Español".to_string()
    } else if lower.starts_with("en") || lower.starts_with("eng") {
        "Inglés".to_string()
    } else if lower.starts_with("fr") || lower.starts_with("fra") {
        "Francés".to_string()
    } else if lower.starts_with("de") || lower.starts_with("deu") {
        "Alemán".to_string()
    } else if lower.starts_with("it") || lower.starts_with("ita") {
        "Italiano".to_string()
    } else if lower.starts_with("pt") || lower.starts_with("por") {
        "Portugués".to_string()
    } else if lower.starts_with("ja") || lower.starts_with("jpn") {
        "Japonés".to_string()
    } else if lower.starts_with("ru") || lower.starts_with("rus") {
        "Ruso".to_string()
    } else if lower.starts_with("zh") || lower.starts_with("zho") {
        "Chino".to_string()
    } else if lower.starts_with("ko") || lower.starts_with("kor") {
        "Coreano".to_string()
    } else if code.is_empty() {
        String::new()
    } else {
        code.to_uppercase()
    }
}

/// Formatea el título legible de una pista combinando idioma y descripción.
pub fn format_track_title(track: &TrackInfo) -> String {
    let lang_display = if !track.lang.is_empty() {
        lang_code_to_name(&track.lang)
    } else {
        String::new()
    };

    if !track.title.is_empty() {
        if !lang_display.is_empty()
            && !track
                .title
                .to_lowercase()
                .contains(&lang_display.to_lowercase())
        {
            format!("{lang_display} - {}", track.title)
        } else {
            track.title.clone()
        }
    } else if !lang_display.is_empty() {
        format!("Pista {}: {lang_display}", track.id)
    } else {
        format!("Pista {}", track.id)
    }
}

/// Obtiene la lista actual de pistas de un tipo ("audio" o "sub") desde el motor mpv.
pub fn get_available_tracks(video_area: &VideoGLArea, track_type: &str) -> Vec<TrackInfo> {
    let mut tracks = Vec::new();
    let count = video_area
        .get_property_str("track-list/count")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    for i in 0..count {
        let t_type = video_area
            .get_property_str(&format!("track-list/{i}/type"))
            .unwrap_or_default();
        if t_type != track_type {
            continue;
        }

        let id = video_area
            .get_property_str(&format!("track-list/{i}/id"))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let title = video_area
            .get_property_str(&format!("track-list/{i}/title"))
            .unwrap_or_default();
        let lang = video_area
            .get_property_str(&format!("track-list/{i}/lang"))
            .unwrap_or_default();
        let selected = video_area
            .get_property_bool(&format!("track-list/{i}/selected"))
            .unwrap_or(false);
        let external = video_area
            .get_property_bool(&format!("track-list/{i}/external"))
            .unwrap_or(false);

        tracks.push(TrackInfo {
            id,
            track_type: t_type,
            title,
            lang,
            selected,
            external,
        });
    }
    tracks
}

/// Crea el botón de menú para pistas de audio con su popover dinámico.
pub fn create_audio_menu_button<FAdd>(
    video_area: VideoGLArea,
    on_add_external: FAdd,
) -> gtk::MenuButton
where
    FAdd: Fn() + Clone + 'static,
{
    let popover = gtk::Popover::builder()
        .position(gtk::PositionType::Top)
        .build();

    let scroll = gtk::ScrolledWindow::builder()
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .max_content_height(380)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    popover.set_child(Some(&scroll));

    // Reconstruir dinámicamente el contenido del popover cada vez que se abre
    {
        let va = video_area.clone();
        let scroll_weak = scroll.downgrade();
        let pop_weak = popover.downgrade();
        let f_add = on_add_external;

        popover.connect_show(move |_| {
            let Some(scr) = scroll_weak.upgrade() else {
                return;
            };

            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(4)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .width_request(280)
                .build();

            let header_lbl = gtk::Label::builder()
                .label("Pistas de audio")
                .halign(gtk::Align::Start)
                .margin_start(4)
                .margin_bottom(4)
                .build();
            header_lbl.add_css_class("heading");
            container.append(&header_lbl);

            let tracks = get_available_tracks(&va, "audio");
            if tracks.is_empty() {
                let empty_lbl = gtk::Label::builder()
                    .label("No se detectaron pistas de audio adicionales")
                    .wrap(true)
                    .halign(gtk::Align::Start)
                    .margin_start(4)
                    .margin_bottom(4)
                    .build();
                empty_lbl.add_css_class("dim-label");
                container.append(&empty_lbl);
            } else {
                for track in tracks {
                    let title_text = format_track_title(&track);
                    let row_btn = gtk::Button::builder()
                        .halign(gtk::Align::Fill)
                        .build();
                    row_btn.add_css_class("flat");
                    row_btn.add_css_class("track-row");

                    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    if track.selected {
                        let check_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                        check_icon.add_css_class("accent");
                        row_box.append(&check_icon);
                        row_btn.add_css_class("selected");
                    } else {
                        let space = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        space.set_width_request(16);
                        row_box.append(&space);
                    }

                    let lbl = gtk::Label::builder()
                        .label(&title_text)
                        .halign(gtk::Align::Start)
                        .hexpand(true)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();
                    row_box.append(&lbl);

                    if track.external {
                        let ext_lbl = gtk::Label::builder()
                            .label("Externa")
                            .halign(gtk::Align::End)
                            .build();
                        ext_lbl.add_css_class("dim-label");
                        row_box.append(&ext_lbl);
                    }

                    row_btn.set_child(Some(&row_box));

                    let v_clone = va.clone();
                    let t_id = track.id;
                    let was_selected = track.selected;
                    let p_weak = pop_weak.clone();

                    row_btn.connect_clicked(move |_| {
                        let selected_id = if was_selected {
                            "no".to_string()
                        } else {
                            t_id.to_string()
                        };
                        v_clone.command(&["set", "aid", &selected_id]);
                        if let Some(pop) = p_weak.upgrade() {
                            pop.popdown();
                        }
                    });
                    container.append(&row_btn);
                }
            }

            let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
            container.append(&sep);

            let add_btn = gtk::Button::builder()
                .label("Añadir pista de audio externa…")
                .icon_name("cine-list-add-symbolic")
                .halign(gtk::Align::Fill)
                .build();
            add_btn.add_css_class("flat");

            let add_cb = f_add.clone();
            let p_weak = pop_weak.clone();
            add_btn.connect_clicked(move |_| {
                if let Some(pop) = p_weak.upgrade() {
                    pop.popdown();
                }
                add_cb();
            });
            container.append(&add_btn);

            scr.set_child(Some(&container));
        });
    }

    let menu_btn = gtk::MenuButton::builder()
        .icon_name("cine-audio-symbolic")
        .tooltip_text("Pistas de audio")
        .popover(&popover)
        .direction(gtk::ArrowType::Up)
        .build();
    menu_btn.add_css_class("circular");
    menu_btn.add_css_class("flat");

    menu_btn
}

/// Crea el botón de menú para subtítulos con su popover dinámico.
pub fn create_subtitles_menu_button<FAdd>(
    video_area: VideoGLArea,
    on_add_external: FAdd,
) -> gtk::MenuButton
where
    FAdd: Fn() + Clone + 'static,
{
    let popover = gtk::Popover::builder()
        .position(gtk::PositionType::Top)
        .build();

    let scroll = gtk::ScrolledWindow::builder()
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .max_content_height(380)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    popover.set_child(Some(&scroll));

    // Reconstruir dinámicamente el contenido del popover cada vez que se abre
    {
        let va = video_area.clone();
        let scroll_weak = scroll.downgrade();
        let pop_weak = popover.downgrade();
        let f_add = on_add_external;

        popover.connect_show(move |_| {
            let Some(scr) = scroll_weak.upgrade() else {
                return;
            };

            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(4)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .width_request(280)
                .build();

            let header_lbl = gtk::Label::builder()
                .label("Subtítulos")
                .halign(gtk::Align::Start)
                .margin_start(4)
                .margin_bottom(4)
                .build();
            header_lbl.add_css_class("heading");
            container.append(&header_lbl);

            let tracks = get_available_tracks(&va, "sub");
            let sub_visibility = va
                .get_property_str("sub-visibility")
                .unwrap_or_else(|| "yes".to_string());
            let current_sid = va
                .get_property_str("sid")
                .unwrap_or_else(|| "no".to_string());
            let subs_disabled = sub_visibility == "no" || current_sid == "no";

            // Opción para desactivar subtítulos
            let off_btn = gtk::Button::builder()
                .halign(gtk::Align::Fill)
                .build();
            off_btn.add_css_class("flat");
            off_btn.add_css_class("track-row");

            let off_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            if subs_disabled {
                let check_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                check_icon.add_css_class("accent");
                off_box.append(&check_icon);
                off_btn.add_css_class("selected");
            } else {
                let space = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                space.set_width_request(16);
                off_box.append(&space);
            }

            let off_lbl = gtk::Label::builder()
                .label("Desactivar subtítulos")
                .halign(gtk::Align::Start)
                .hexpand(true)
                .build();
            off_box.append(&off_lbl);
            off_btn.set_child(Some(&off_box));

            {
                let v_clone = va.clone();
                let p_weak = pop_weak.clone();
                off_btn.connect_clicked(move |_| {
                    v_clone.command(&["set", "sub-visibility", "no"]);
                    v_clone.command(&["set", "sid", "no"]);
                    if let Some(pop) = p_weak.upgrade() {
                        pop.popdown();
                    }
                });
            }
            container.append(&off_btn);

            let sep1 = gtk::Separator::new(gtk::Orientation::Horizontal);
            container.append(&sep1);

            if tracks.is_empty() {
                let empty_lbl = gtk::Label::builder()
                    .label("No se detectaron pistas de subtítulos en el video")
                    .wrap(true)
                    .halign(gtk::Align::Start)
                    .margin_start(4)
                    .margin_bottom(4)
                    .build();
                empty_lbl.add_css_class("dim-label");
                container.append(&empty_lbl);
            } else {
                for track in tracks {
                    let title_text = format_track_title(&track);
                    let row_btn = gtk::Button::builder()
                        .halign(gtk::Align::Fill)
                        .build();
                    row_btn.add_css_class("flat");
                    row_btn.add_css_class("track-row");

                    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    let is_active = !subs_disabled && track.selected;
                    if is_active {
                        let check_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                        check_icon.add_css_class("accent");
                        row_box.append(&check_icon);
                        row_btn.add_css_class("selected");
                    } else {
                        let space = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        space.set_width_request(16);
                        row_box.append(&space);
                    }

                    let lbl = gtk::Label::builder()
                        .label(&title_text)
                        .halign(gtk::Align::Start)
                        .hexpand(true)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();
                    row_box.append(&lbl);

                    if track.external {
                        let ext_lbl = gtk::Label::builder()
                            .label("Externa")
                            .halign(gtk::Align::End)
                            .build();
                        ext_lbl.add_css_class("dim-label");
                        row_box.append(&ext_lbl);
                    }

                    row_btn.set_child(Some(&row_box));

                    let v_clone = va.clone();
                    let t_id = track.id;
                    let was_active = is_active;
                    let p_weak = pop_weak.clone();

                    row_btn.connect_clicked(move |_| {
                        if was_active {
                            v_clone.command(&["set", "sub-visibility", "no"]);
                            v_clone.command(&["set", "sid", "no"]);
                        } else {
                            v_clone.command(&["set", "sub-visibility", "yes"]);
                            v_clone.command(&["set", "sid", &t_id.to_string()]);
                        }
                        if let Some(pop) = p_weak.upgrade() {
                            pop.popdown();
                        }
                    });
                    container.append(&row_btn);
                }
            }

            let sep2 = gtk::Separator::new(gtk::Orientation::Horizontal);
            container.append(&sep2);

            let add_btn = gtk::Button::builder()
                .label("Añadir archivo de subtítulos…")
                .icon_name("cine-list-add-symbolic")
                .halign(gtk::Align::Fill)
                .build();
            add_btn.add_css_class("flat");

            let add_cb = f_add.clone();
            let p_weak = pop_weak.clone();
            add_btn.connect_clicked(move |_| {
                if let Some(pop) = p_weak.upgrade() {
                    pop.popdown();
                }
                add_cb();
            });
            container.append(&add_btn);

            scr.set_child(Some(&container));
        });
    }

    let menu_btn = gtk::MenuButton::builder()
        .icon_name("cine-subtitles-symbolic")
        .tooltip_text("Subtítulos")
        .popover(&popover)
        .direction(gtk::ArrowType::Up)
        .build();
    menu_btn.add_css_class("circular");
    menu_btn.add_css_class("flat");

    menu_btn
}
