// options.rs
//
// Menú desplegable de opciones de video, subtítulos, rotación, zoom y velocidad.
// Equivalente modular a `options.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::mpv_gl_area::VideoGLArea;

/// Valores de transformación de video y velocidad de reproducción.
#[derive(Clone, Debug)]
pub struct PlaybackOptions {
    pub speed: f64,
    pub video_rotate: i32,
    pub video_zoom: f64,
    pub contrast: i32,
    pub brightness: i32,
    pub gamma: i32,
    pub saturation: i32,
    pub hue: i32,
    pub sub_delay: f64,
    pub audio_delay: f64,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            speed: 1.0,
            video_rotate: 0,
            video_zoom: 0.0,
            contrast: 0,
            brightness: 0,
            gamma: 0,
            saturation: 0,
            hue: 0,
            sub_delay: 0.0,
            audio_delay: 0.0,
        }
    }
}

/// Construye el botón de opciones de video con su popover interactivo.
pub fn create_options_menu_button(
    video_area: VideoGLArea,
    options: Rc<RefCell<PlaybackOptions>>,
    zoom_indicator: gtk::Label,
    zoom_indicator_timer: Rc<RefCell<Option<glib::SourceId>>>,
) -> gtk::MenuButton {
    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .width_request(280)
        .build();

    // Botón "Restablecer todo"
    let reset_all_btn = gtk::Button::builder()
        .label("Restablecer todo")
        .hexpand(true)
        .build();
    content_box.append(&reset_all_btn);

    // --- RELACIÓN DE ASPECTO ---
    let aspect_list = [
        "Original", "16:9", "4:3", "1:1", "16:10", "2.00:1", "2.21:1", "2.35:1", "2.39:1", "5:4",
    ];
    let aspect_str_list = gtk::StringList::new(&aspect_list);
    let aspect_dropdown = gtk::DropDown::builder()
        .model(&aspect_str_list)
        .selected(0)
        .width_request(110)
        .halign(gtk::Align::End)
        .build();

    let aspect_reset_btn = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Restablecer relación de aspecto")
        .build();
    aspect_reset_btn.add_css_class("flat");

    let aspect_row = make_option_row(
        &aspect_reset_btn,
        "Relación de aspecto",
        &aspect_dropdown,
    );
    content_box.append(&aspect_row);

    {
        let va = video_area.clone();
        let dropdown = aspect_dropdown.clone();
        aspect_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected();
            let aspect = match idx {
                1 => "16:9",
                2 => "4:3",
                3 => "1:1",
                4 => "16:10",
                5 => "2.00:1",
                6 => "2.21:1",
                7 => "2.35:1",
                8 => "2.39:1",
                9 => "5:4",
                _ => "-1",
            };
            va.set_aspect_ratio(aspect);
        });

        let dd_clone = dropdown;
        aspect_reset_btn.connect_clicked(move |_| {
            dd_clone.set_selected(0);
        });
    }

    // --- ROTACIÓN ---
    let rotate_reset_btn = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Restablecer rotación")
        .build();
    rotate_reset_btn.add_css_class("flat");

    let rotate_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let rot_left_btn = gtk::Button::builder()
        .icon_name("object-rotate-left-symbolic")
        .tooltip_text("Rotar 90° izquierda")
        .width_request(44)
        .build();
    let rot_right_btn = gtk::Button::builder()
        .icon_name("object-rotate-right-symbolic")
        .tooltip_text("Rotar 90° derecha")
        .width_request(44)
        .build();
    rotate_box.append(&rot_left_btn);
    rotate_box.append(&rot_right_btn);

    let rotate_row = make_option_row(&rotate_reset_btn, "Rotar", &rotate_box);
    content_box.append(&rotate_row);

    {
        let va = video_area.clone();
        let opts = options.clone();
        rot_left_btn.connect_clicked(move |_| {
            let mut o = opts.borrow_mut();
            o.video_rotate = (o.video_rotate - 90 + 360) % 360;
            va.set_video_rotate(o.video_rotate);
        });

        let va2 = video_area.clone();
        let opts2 = options.clone();
        rot_right_btn.connect_clicked(move |_| {
            let mut o = opts2.borrow_mut();
            o.video_rotate = (o.video_rotate + 90) % 360;
            va2.set_video_rotate(o.video_rotate);
        });

        let va3 = video_area.clone();
        let opts3 = options.clone();
        rotate_reset_btn.connect_clicked(move |_| {
            let mut o = opts3.borrow_mut();
            o.video_rotate = 0;
            va3.set_video_rotate(0);
        });
    }

    // --- ZOOM ---
    let (zoom_row, zoom_spin, zoom_reset_btn) = make_spin_row(
        "Zoom",
        "Restablecer zoom",
        0.0,
        2.0,
        0.05,
        2,
        0.0,
    );
    content_box.append(&zoom_row);

    {
        let va = video_area.clone();
        let opts = options.clone();
        let indicator = zoom_indicator.clone();
        let indicator_timer = zoom_indicator_timer.clone();
        let spin = zoom_spin.clone();
        zoom_spin.connect_value_changed(move |s| {
            opts.borrow_mut().video_zoom = s.value();
            va.set_video_zoom(s.value());
            crate::window::update_zoom_indicator(&indicator, &indicator_timer, s.value());
        });
        zoom_reset_btn.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- CONTRASTE ---
    let (contrast_row, contrast_spin, contrast_reset) = make_spin_row(
        "Contraste",
        "Restablecer contraste",
        -100.0,
        100.0,
        1.0,
        0,
        0.0,
    );
    content_box.append(&contrast_row);

    {
        let va = video_area.clone();
        let spin = contrast_spin.clone();
        contrast_spin.connect_value_changed(move |s| {
            va.set_contrast(s.value() as i32);
        });
        contrast_reset.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- BRILLO ---
    let (bright_row, bright_spin, bright_reset) = make_spin_row(
        "Brillo",
        "Restablecer brillo",
        -100.0,
        100.0,
        1.0,
        0,
        0.0,
    );
    content_box.append(&bright_row);

    {
        let va = video_area.clone();
        let spin = bright_spin.clone();
        bright_spin.connect_value_changed(move |s| {
            va.set_brightness(s.value() as i32);
        });
        bright_reset.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- SATURACIÓN ---
    let (sat_row, sat_spin, sat_reset) = make_spin_row(
        "Saturación",
        "Restablecer saturación",
        -100.0,
        100.0,
        1.0,
        0,
        0.0,
    );
    content_box.append(&sat_row);

    {
        let va = video_area.clone();
        let spin = sat_spin.clone();
        sat_spin.connect_value_changed(move |s| {
            va.set_saturation(s.value() as i32);
        });
        sat_reset.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- DESFASE DE SUBTÍTULOS ---
    let (sub_row, sub_spin, sub_reset) = make_spin_row(
        "Desfase subtítulos",
        "Restablecer desfase de subtítulos",
        -999.0,
        999.0,
        0.1,
        1,
        0.0,
    );
    content_box.append(&sub_row);

    {
        let va = video_area.clone();
        let opts = options.clone();
        let spin = sub_spin.clone();
        sub_spin.connect_value_changed(move |s| {
            opts.borrow_mut().sub_delay = s.value();
            va.set_sub_delay(s.value());
        });
        sub_reset.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- DESFASE DE AUDIO ---
    let (aud_row, aud_spin, aud_reset) = make_spin_row(
        "Desfase audio",
        "Restablecer desfase de audio",
        -999.0,
        999.0,
        0.1,
        1,
        0.0,
    );
    content_box.append(&aud_row);

    {
        let va = video_area.clone();
        let opts = options.clone();
        let spin = aud_spin.clone();
        aud_spin.connect_value_changed(move |s| {
            opts.borrow_mut().audio_delay = s.value();
            va.set_audio_delay(s.value());
        });
        aud_reset.connect_clicked(move |_| {
            spin.set_value(0.0);
        });
    }

    // --- VELOCIDAD DE REPRODUCCIÓN ---
    let (speed_row, speed_spin, speed_reset) = make_spin_row(
        "Velocidad",
        "Restablecer velocidad",
        0.25,
        10.0,
        0.25,
        2,
        1.0,
    );
    content_box.append(&speed_row);

    {
        let va = video_area.clone();
        let opts = options.clone();
        let spin = speed_spin.clone();
        speed_spin.connect_value_changed(move |s| {
            opts.borrow_mut().speed = s.value();
            va.set_speed(s.value());
        });
        speed_reset.connect_clicked(move |_| {
            spin.set_value(1.0);
        });
    }

    // Reset All callback
    {
        let dd = aspect_dropdown.clone();
        let r_btn = rotate_reset_btn.clone();
        let z_spin = zoom_spin.clone();
        let c_spin = contrast_spin.clone();
        let b_spin = bright_spin.clone();
        let sat_s = sat_spin.clone();
        let s_spin = sub_spin.clone();
        let a_spin = aud_spin.clone();
        let sp_spin = speed_spin.clone();

        reset_all_btn.connect_clicked(move |_| {
            dd.set_selected(0);
            r_btn.emit_clicked();
            z_spin.set_value(0.0);
            c_spin.set_value(0.0);
            b_spin.set_value(0.0);
            sat_s.set_value(0.0);
            s_spin.set_value(0.0);
            a_spin.set_value(0.0);
            sp_spin.set_value(1.0);
        });
    }

    let scrolled = gtk::ScrolledWindow::builder()
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .max_content_height(480)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&content_box)
        .build();

    let popover = gtk::Popover::builder().child(&scrolled).build();
    {
        let zoom_spin = zoom_spin.clone();
        let options = options.clone();
        popover.connect_show(move |_| {
            let zoom = options.borrow().video_zoom;
            zoom_spin.set_value(zoom);
        });
    }

    let menu_btn = gtk::MenuButton::builder()
        .icon_name("cine-options-symbolic")
        .tooltip_text("Opciones de video")
        .popover(&popover)
        .direction(gtk::ArrowType::Up)
        .build();
    menu_btn.add_css_class("circular");
    menu_btn.add_css_class("flat");

    menu_btn
}

fn make_option_row(
    reset_btn: &gtk::Button,
    label_text: &str,
    right_widget: &impl IsA<gtk::Widget>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    left_box.append(reset_btn);

    let lbl = gtk::Label::builder()
        .label(label_text)
        .halign(gtk::Align::Start)
        .build();
    lbl.add_css_class("heading");
    left_box.append(&lbl);

    row.append(&left_box);

    let spacer = gtk::Separator::new(gtk::Orientation::Horizontal);
    spacer.set_hexpand(true);
    spacer.set_opacity(0.0);
    row.append(&spacer);

    row.append(right_widget);
    row
}

fn make_spin_row(
    label: &str,
    reset_tooltip: &str,
    lower: f64,
    upper: f64,
    step: f64,
    digits: u32,
    default_val: f64,
) -> (gtk::Box, gtk::SpinButton, gtk::Button) {
    let reset_btn = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text(reset_tooltip)
        .build();
    reset_btn.add_css_class("flat");

    let adj = gtk::Adjustment::new(default_val, lower, upper, step, step * 5.0, 0.0);
    let spin = gtk::SpinButton::builder()
        .adjustment(&adj)
        .digits(digits)
        .climb_rate(1.0)
        .width_request(88)
        .halign(gtk::Align::End)
        .build();
    spin.add_css_class("cine-spinbutton");

    let row = make_option_row(&reset_btn, label, &spin);
    (row, spin, reset_btn)
}
