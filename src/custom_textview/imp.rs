use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use gtk::glib;
use gtk::{CompositeTemplate, TemplateChild};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use webkit2gtk::WebView;
use webkit2gtk::prelude::WebViewExt;

#[derive(CompositeTemplate)]
#[template(resource = "/org/md-wr/com/text-editor.ui")]
pub struct CustomTextView {
    #[template_child]
    pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

    #[template_child]
    pub text_view: TemplateChild<gtk::TextView>,

    #[template_child]
    pub placeholder_label: TemplateChild<gtk::Label>,

    #[template_child]
    pub word_count_label: TemplateChild<gtk::Label>,

    #[template_child]
    pub char_count_label: TemplateChild<gtk::Label>,

    #[template_child]
    pub save_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub clear_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,

    // Navigation panel components
    #[template_child]
    pub nav_toggle: TemplateChild<gtk::ToggleButton>,

    #[template_child]
    pub nav_revealer: TemplateChild<gtk::Revealer>,

    #[template_child]
    pub main_paned: TemplateChild<gtk::Paned>,

    #[template_child]
    pub web_view: TemplateChild<WebView>,

    settings: Option<Settings>,
    config_dir: PathBuf,
    settings_key: RefCell<Option<String>>,
    auto_save: RefCell<bool>,
    is_loading: RefCell<bool>,
    nav_visible: RefCell<bool>,
    paned_position: RefCell<i32>,
}

impl Default for CustomTextView {
    fn default() -> Self {
        let config_dir = glib::user_config_dir().join("md-wr");

        // Try to create settings, fallback to None if it fails
        let settings = Settings::new("org.md-wr.com");

        let instance = Self {
            scrolled_window: TemplateChild::default(),
            text_view: TemplateChild::default(),
            web_view: TemplateChild::default(),
            word_count_label: TemplateChild::default(),
            char_count_label: TemplateChild::default(),
            save_button: TemplateChild::default(),
            clear_button: TemplateChild::default(),
            placeholder_label: TemplateChild::default(),
            title_label: TemplateChild::default(),
            nav_toggle: TemplateChild::default(),
            nav_revealer: TemplateChild::default(),
            main_paned: TemplateChild::default(),
            settings: Some(settings),
            config_dir,
            settings_key: RefCell::new(None),
            auto_save: RefCell::new(false),
            is_loading: RefCell::new(false),
            nav_visible: RefCell::new(false),
            paned_position: RefCell::new(250),
        };
        instance.ensure_config_dir(); // Ensure config dir early
        instance
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CustomTextView {
    const NAME: &'static str = "CustomTextView";
    type Type = super::CustomTextView;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl CustomTextView {
    fn ensure_config_dir(&self) {
        if let Err(e) = fs::create_dir_all(&self.config_dir) {
            eprintln!("Failed to create config directory: {}", e);
        }
    }

    fn get_config_value(&self, key: &str, default: &str) -> String {
        if let Some(ref settings) = self.settings {
            // Try to get the value, fall back to default if key doesn't exist or operation fails
            match key {
                "navigation-panel-visible" => {
                    settings.boolean("navigation-panel-visible").to_string()
                }
                "paned-position" => settings.int("paned-position").to_string(),
                _ => settings.string(key).to_string(),
            }
        } else {
            let config_file = self.config_dir.join(format!("{}.txt", key));
            fs::read_to_string(config_file).unwrap_or_else(|_| default.to_string())
        }
    }

    fn set_config_value(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref settings) = self.settings {
            // Try to set the value directly, fall back to file if it fails
            let result = match key {
                "navigation-panel-visible" => {
                    let bool_val = value.parse::<bool>()?;
                    settings
                        .set_boolean(key, bool_val)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                }
                "paned-position" => {
                    let int_val = value.parse::<i32>()?;
                    settings
                        .set_int(key, int_val)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                }
                _ => settings
                    .set_string(key, value)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            };

            if let Err(e) = result {
                eprintln!(
                    "GSettings operation failed for key '{}': {}. Falling back to file",
                    key, e
                );
                self.ensure_config_dir();
                let config_file = self.config_dir.join(format!("{}.txt", key));
                fs::write(config_file, value)?;
            }
        } else {
            self.ensure_config_dir();
            let config_file = self.config_dir.join(format!("{}.txt", key));
            fs::write(config_file, value)?;
        }
        Ok(())
    }

    fn on_save_clicked(&self) {
        println!("Save button clicked!");
        let obj = self.obj();
        obj.save_to_settings();
        println!("Text saved to settings!");
    }

    fn on_clear_clicked(&self) {
        println!("Clear button clicked!");
        let obj = self.obj();
        obj.clear_text();
        self.update_counts();
        if *self.auto_save.borrow() && self.settings_key.borrow().is_some() {
            self.save_to_settings();
        }
        println!("Text cleared!");
    }

    fn on_nav_toggle_clicked(&self) {
        let is_active = self.nav_toggle.is_active();
        println!("Navigation toggle clicked! Active: {}", is_active);

        // Update internal state
        *self.nav_visible.borrow_mut() = is_active;

        if is_active {
            // Show: Add child, restore position, animate in
            self.nav_revealer.set_reveal_child(false); // Start hidden for animation
            self.main_paned.set_end_child(Some(&*self.nav_revealer));
            self.main_paned.set_position(*self.paned_position.borrow());
            self.nav_revealer.set_reveal_child(true); // Trigger slide-in
            self.nav_toggle.set_icon_name("sidebar-show-symbolic-rtl");
            self.nav_toggle
                .set_tooltip_text(Some("Hide Navigation Panel"));
        } else {
            // Hide: Save position, animate out, then remove child after transition
            *self.paned_position.borrow_mut() = self.main_paned.position();
            self.nav_revealer.set_reveal_child(false); // Trigger slide-out
            let imp_weak = self.obj().downgrade();
            glib::timeout_add_local_once(Duration::from_millis(200), move || {
                if let Some(imp) = imp_weak.upgrade() {
                    let imp = imp.imp();
                    imp.main_paned.set_end_child(None::<&gtk::Widget>);
                }
            });
            self.nav_toggle.set_icon_name("view-dual-symbolic");
            self.nav_toggle
                .set_tooltip_text(Some("Show Navigation Panel"));
        }

        // Save state
        if let Err(e) = self.set_config_value("navigation-panel-visible", &is_active.to_string()) {
            eprintln!("Failed to save navigation panel state: {}", e);
        }
        if let Err(e) =
            self.set_config_value("paned-position", &self.paned_position.borrow().to_string())
        {
            eprintln!("Failed to save paned position: {}", e);
        }

        println!(
            "Navigation panel state saved: visible={}, position={}",
            is_active,
            *self.paned_position.borrow()
        );
    }

    pub fn load_navigation_state(&self) {
        // Load navigation panel visibility
        let nav_visible_str = self.get_config_value("navigation-panel-visible", "false");
        let nav_visible = nav_visible_str.parse::<bool>().unwrap_or(false);

        *self.nav_visible.borrow_mut() = nav_visible;
        self.nav_toggle.set_active(nav_visible);

        // Update icon based on loaded state
        if nav_visible {
            self.nav_toggle.set_icon_name("sidebar-show-symbolic-rtl");
            self.nav_toggle
                .set_tooltip_text(Some("Hide Navigation Panel"));
            self.nav_revealer.set_reveal_child(true);
            self.main_paned.set_end_child(Some(&*self.nav_revealer)); // Ensure added if visible
        } else {
            self.nav_toggle.set_icon_name("view-dual-symbolic");
            self.nav_toggle
                .set_tooltip_text(Some("Show Navigation Panel"));
            self.nav_revealer.set_reveal_child(false);
            // Ensure removed if hidden
        }

        // Load paned position
        let position_str = self.get_config_value("paned-position", "250");
        let saved_position = position_str.parse::<i32>().unwrap_or(250);
        if saved_position > 0 {
            *self.paned_position.borrow_mut() = saved_position;
            if nav_visible {
                self.main_paned.set_position(saved_position);
            }
        }

        println!(
            "Navigation state loaded: visible={}, position={}",
            nav_visible, saved_position
        );
    }

    pub fn clear_text(&self) {
        println!("Clearing text buffer");
        let buffer = self.text_view.buffer();
        buffer.set_text("");
        self.text_view.queue_draw();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        let text = buffer.text(&start, &end, false);
        println!("Buffer after clear: '{}'", text);
    }

    fn update_counts(&self) {
        let buffer = self.text_view.buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        let text = buffer.text(&start, &end, false);
        println!("Updating counts: text='{}'", text);

        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();

        self.char_count_label
            .set_text(&format!("Characters: {}", char_count));
        self.word_count_label
            .set_text(&format!("Words: {}", word_count));

        self.update_placeholder_visibility(&text);
    }

    fn update_placeholder_visibility(&self, text: &str) {
        if text.trim().is_empty() {
            self.placeholder_label.set_visible(true);
        } else {
            self.placeholder_label.set_visible(false);
        }
    }

    pub fn set_settings_key(&self, key: &str) {
        *self.settings_key.borrow_mut() = Some(key.to_string());
        self.load_from_settings();
    }

    pub fn save_to_settings(&self) {
        if *self.is_loading.borrow() {
            return;
        }
        if let Some(key) = self.settings_key.borrow().as_ref() {
            let buffer = self.text_view.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            if let Err(e) = self.set_config_value(key, &text) {
                eprintln!("Failed to save text to settings: {}", e);
            } else {
                println!("Text saved to key: {}", key);
            }
        } else {
            eprintln!("No settings key configured for this text view");
        }
    }

    pub fn load_from_settings(&self) {
        if let Some(key) = self.settings_key.borrow().as_ref() {
            *self.is_loading.borrow_mut() = true;
            let saved_text = self.get_config_value(key, "");
            let buffer = self.text_view.buffer();
            buffer.set_text(&saved_text);
            self.update_counts();
            *self.is_loading.borrow_mut() = false;
            println!(
                "Loaded text from key: {} (length: {})",
                key,
                saved_text.len()
            );
        }
    }

    pub fn set_auto_save(&self, auto_save: bool) {
        *self.auto_save.borrow_mut() = auto_save;
    }
}

impl ObjectImpl for CustomTextView {
    fn constructed(&self) {
        self.parent_constructed();

        // the template child WebView
        self.web_view.load_uri("https://www.gtk.org");
        self.web_view.set_vexpand(true);
        self.web_view.set_hexpand(true);

        self.web_view
            .connect_context_menu(|_web_view, _context_menu, _event| true);

        self.web_view.connect_load_changed(|_view, event| {
            println!("WebView load event: {:?}", event);
        });

        // Set up the text view properties
        self.text_view.set_wrap_mode(gtk::WrapMode::None);
        self.text_view.set_accepts_tab(true);
        self.text_view.set_monospace(false);
        self.text_view.set_left_margin(8);
        self.text_view.set_right_margin(8);
        self.text_view.set_top_margin(8);
        self.text_view.set_bottom_margin(8);
        self.text_view.add_css_class("monospace");

        // Set up scrolled window properties
        self.scrolled_window
            .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        self.scrolled_window.set_min_content_height(150);
        self.scrolled_window.add_css_class("card");

        // Set up placeholder initially
        self.placeholder_label.set_visible(true);
        self.placeholder_label.set_sensitive(false);
        self.placeholder_label.add_css_class("dim-label");

        // Add CSS classes for other widgets
        self.title_label.add_css_class("heading");
        self.save_button.add_css_class("flat");
        self.clear_button.add_css_class("flat");
        self.clear_button.add_css_class("destructive-action");
        self.char_count_label.add_css_class("caption");
        self.char_count_label.add_css_class("dim-label");
        self.word_count_label.add_css_class("caption");
        self.word_count_label.add_css_class("dim-label");

        // Manually connect button signals
        self.save_button.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.on_save_clicked();
            }
        ));

        self.clear_button.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.on_clear_clicked();
            }
        ));

        // Connect navigation toggle button
        self.nav_toggle.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.on_nav_toggle_clicked();
            }
        ));

        // Connect buffer changed signal to update counts and auto-save
        let buffer = self.text_view.buffer();
        buffer.connect_changed(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.update_counts();
                if *imp.auto_save.borrow() && imp.settings_key.borrow().is_some() {
                    imp.save_to_settings();
                }
            }
        ));

        // Connect focus events to handle placeholder
        let focus_controller = gtk::EventControllerFocus::new();
        focus_controller.connect_enter(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.placeholder_label.set_visible(false);
            }
        ));
        focus_controller.connect_leave(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                let buffer = imp.text_view.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, false);
                imp.update_placeholder_visibility(&text);
            }
        ));
        self.text_view.add_controller(focus_controller);

        // Load navigation state after everything is set up
        self.load_navigation_state();

        // Initial count update
        self.update_counts();
    }

    fn dispose(&self) {
        if *self.auto_save.borrow() {
            self.save_to_settings();
        }

        // Save final navigation state
        let nav_visible = *self.nav_visible.borrow();
        let position = *self.paned_position.borrow();
        let _ = self.set_config_value("navigation-panel-visible", &nav_visible.to_string());
        let _ = self.set_config_value("paned-position", &position.to_string());

        self.scrolled_window.unparent();
    }
}

impl WidgetImpl for CustomTextView {}
