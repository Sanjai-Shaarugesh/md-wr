use adw::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

mod imp;

glib::wrapper! {
    pub struct CustomTextView(ObjectSubclass<imp::CustomTextView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CustomTextView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    /// Create a new CustomTextView with a specific settings key for auto-saving
    pub fn with_settings_key(key: &str) -> Self {
        let widget = Self::new();
        widget.set_settings_key(key);
        widget.set_auto_save(true); // Enable auto-save by default when key is provided
        widget
    }

    pub fn set_placeholder_text(&self, text: &str) {
        let imp = self.imp();
        imp.placeholder_label.set_text(text);
    }

    pub fn get_text(&self) -> String {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        buffer.text(&start, &end, false).to_string()
    }

    pub fn set_text(&self, text: &str) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        buffer.set_text(text);
    }

    pub fn clear_text(&self) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        buffer.set_text("");
    }

    /// Set the settings key for saving/loading text content
    pub fn set_settings_key(&self, key: &str) {
        let imp = self.imp();
        imp.set_settings_key(key);
    }

    /// Manually save current text to settings
    pub fn save_to_settings(&self) {
        let imp = self.imp();
        imp.save_to_settings();
    }

    /// Load text from settings
    pub fn load_from_settings(&self) {
        let imp = self.imp();
        imp.load_from_settings();
    }

    /// Enable or disable auto-saving on text changes
    pub fn set_auto_save(&self, auto_save: bool) {
        let imp = self.imp();
        imp.set_auto_save(auto_save);
    }

    pub fn set_monospace(&self, monospace: bool) {
        let imp = self.imp();
        imp.text_view.set_monospace(monospace);
    }

    pub fn set_wrap_mode(&self, wrap_mode: gtk::WrapMode) {
        let imp = self.imp();
        imp.text_view.set_wrap_mode(wrap_mode);
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.get_text().split_whitespace().count()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.get_text().chars().count()
    }

    // Navigation panel methods

    /// Toggle the navigation panel visibility
    pub fn toggle_navigation_panel(&self) {
        let imp = self.imp();
        let current_state = imp.nav_toggle.is_active();
        imp.nav_toggle.set_active(!current_state);
        // The signal handler will take care of the rest
    }

    /// Show the navigation panel
    pub fn show_navigation_panel(&self) {
        let imp = self.imp();
        if !imp.nav_toggle.is_active() {
            imp.nav_toggle.set_active(true);
        }
    }

    /// Hide the navigation panel
    pub fn hide_navigation_panel(&self) {
        let imp = self.imp();
        if imp.nav_toggle.is_active() {
            imp.nav_toggle.set_active(false);
        }
    }

    /// Check if navigation panel is visible
    pub fn is_navigation_panel_visible(&self) -> bool {
        let imp = self.imp();
        imp.nav_revealer.reveals_child()
    }

    /// Set the paned position (distance from left edge to splitter)
    pub fn set_paned_position(&self, position: i32) {
        let imp = self.imp();
        imp.main_paned.set_position(position);
    }

    /// Get the current paned position
    pub fn get_paned_position(&self) -> i32 {
        let imp = self.imp();
        imp.main_paned.position()
    }

    /// Load navigation state from settings
    pub fn load_navigation_state(&self) {
        let imp = self.imp();
        imp.load_navigation_state();
    }

    /// Connect to text changed events
    pub fn connect_text_changed<F: Fn(&Self) + 'static>(&self, f: F) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        buffer.connect_changed(glib::clone!(
            #[weak(rename_to = widget)]
            self,
            move |_| {
                f(&widget);
            }
        ));
    }

    /// Connect to save button clicked events
    pub fn connect_save_clicked<F: Fn(&Self) + 'static>(&self, f: F) {
        let imp = self.imp();
        imp.save_button.connect_clicked(glib::clone!(
            #[weak(rename_to = widget)]
            self,
            move |_| {
                f(&widget);
            }
        ));
    }

    /// Connect to clear button clicked events
    pub fn connect_clear_clicked<F: Fn(&Self) + 'static>(&self, f: F) {
        let imp = self.imp();
        imp.clear_button.connect_clicked(glib::clone!(
            #[weak(rename_to = widget)]
            self,
            move |_| {
                f(&widget);
            }
        ));
    }

    /// Connect to navigation toggle events
    pub fn connect_navigation_toggled<F: Fn(&Self, bool) + 'static>(&self, f: F) {
        let imp = self.imp();
        imp.nav_toggle.connect_toggled(glib::clone!(
            #[weak(rename_to = widget)]
            self,
            move |toggle| {
                let is_active = toggle.is_active();
                f(&widget, is_active);
            }
        ));
    }

    /// Connect to paned position changes
    pub fn connect_paned_position_changed<F: Fn(&Self, i32) + 'static>(&self, f: F) {
        let imp = self.imp();
        imp.main_paned.connect_position_notify(glib::clone!(
            #[weak(rename_to = widget)]
            self,
            move |paned| {
                let position = paned.position();
                f(&widget, position);
            }
        ));
    }
}

impl Default for CustomTextView {
    fn default() -> Self {
        Self::new()
    }
}
