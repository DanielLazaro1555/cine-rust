// CineWindow responsibility module.
use super::*;

impl CineWindow {
    pub fn update_quality_menu(&self, max_height: i32) {
        let quality_menu = gio::Menu::new();
        quality_menu.append(
            Some("Automático (Calidad óptima del video)"),
            Some("win.set-quality('best')"),
        );

        if max_height >= 2100 {
            quality_menu.append(Some("4K UHD (2160p)"), Some("win.set-quality('2160')"));
        }
        if max_height >= 1400 {
            quality_menu.append(Some("1440p QHD (2K)"), Some("win.set-quality('1440')"));
        }
        if max_height >= 1000 {
            quality_menu.append(Some("1080p FHD"), Some("win.set-quality('1080')"));
        }
        if max_height >= 700 {
            quality_menu.append(Some("720p HD"), Some("win.set-quality('720')"));
        }
        if max_height >= 440 {
            quality_menu.append(Some("480p SD"), Some("win.set-quality('480')"));
        }
        if max_height >= 320 {
            quality_menu.append(Some("360p"), Some("win.set-quality('360')"));
        }

        self.quality_menu_btn.set_menu_model(Some(&quality_menu));
    }

    pub fn set_controls_visible(&self, visible: bool) {
        if self.start_page.is_visible() {
            self.headerbar.set_visible(true);
            self.controls_box.set_visible(false);
            return;
        }

        if self.is_locked.get() {
            self.headerbar.set_visible(false);
            self.controls_box.set_visible(false);
            return;
        }

        self.headerbar.set_visible(visible);
        self.controls_box.set_visible(visible);

        if !visible {
            self.cancel_autohide_timer();
        }
    }

    pub fn lock_controls(&self) {
        if self.start_page.is_visible() {
            return;
        }
        self.is_locked.set(true);
        self.headerbar.set_visible(false);
        self.controls_box.set_visible(false);
        self.cancel_autohide_timer();
        self.reset_floating_lock_timer();
    }

    pub fn unlock_controls(&self) {
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.headerbar.set_visible(true);
        self.controls_box.set_visible(true);
        self.reset_autohide_timer();
    }

    pub fn toggle_lock(&self) {
        if self.is_locked.get() {
            self.unlock_controls();
        } else {
            self.lock_controls();
        }
    }

    pub fn cancel_floating_lock_timer(&self) {
        if let Some(source_id) = self.lock_timer_id.borrow_mut().take() {
            source_id.remove();
        }
    }

    pub fn reset_floating_lock_timer(&self) {
        self.cancel_floating_lock_timer();
        if !self.is_locked.get() {
            return;
        }

        self.floating_lock_btn.set_visible(true);

        let this = self.clone();
        let source_id = glib::timeout_add_local(Duration::from_secs(5), move || {
            if this.is_locked.get() {
                this.floating_lock_btn.set_visible(false);
            }
            *this.lock_timer_id.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *self.lock_timer_id.borrow_mut() = Some(source_id);
    }

    pub fn cancel_autohide_timer(&self) {
        if let Some(source_id) = self.autohide_timer_id.borrow_mut().take() {
            source_id.remove();
        }
    }

    pub fn reset_autohide_timer(&self) {
        self.cancel_autohide_timer();
        if self.start_page.is_visible() {
            return;
        }

        let this = self.clone();
        let source_id = glib::timeout_add_local(Duration::from_secs(3), move || {
            if this.is_playing.get() && this.window.visible_dialog().is_none() {
                this.set_controls_visible(false);
            }
            *this.autohide_timer_id.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *self.autohide_timer_id.borrow_mut() = Some(source_id);
    }

}
