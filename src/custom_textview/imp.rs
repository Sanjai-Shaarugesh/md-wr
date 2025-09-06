use std::cell::RefCell;
use gtk::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{CompositeTemplate, TemplateChild};
use gio::Settings;

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
    
    settings: Settings,
    settings_key: RefCell<Option<String>>,
    auto_save: RefCell<bool>,
    is_loading: RefCell<bool>,
}

impl Default for CustomTextView {
    fn default() -> Self {
        Self {
            scrolled_window: TemplateChild::default(),
            text_view: TemplateChild::default(),
            word_count_label: TemplateChild::default(),
            char_count_label: TemplateChild::default(),
            save_button: TemplateChild::default(),
            clear_button: TemplateChild::default(),
            placeholder_label: TemplateChild::default(),
            title_label: TemplateChild::default(),
            settings: Settings::new("org.gtk-sanjai.textview"),
            settings_key: RefCell::new(None),
            auto_save: RefCell::new(false),
            is_loading: RefCell::new(false),
        }
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
        
        self.char_count_label.set_text(&format!("Characters: {}", char_count));
        self.word_count_label.set_text(&format!("Words: {}", word_count));
        
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
            if let Err(e) = self.settings.set_string(key, &text) {
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
            let saved_text = self.settings.string(key);
            let buffer = self.text_view.buffer();
            buffer.set_text(&saved_text);
            self.update_counts();
            *self.is_loading.borrow_mut() = false;
            println!("Loaded text from key: {} (length: {})", key, saved_text.len());
        }
    }
    
    pub fn set_auto_save(&self, auto_save: bool) {
        *self.auto_save.borrow_mut() = auto_save;
    }
}

impl ObjectImpl for CustomTextView {
    fn constructed(&self) {
        self.parent_constructed();
        
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
        self.scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
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
        
        // Initial count update
        self.update_counts();
    }
    
    fn dispose(&self) {
        if *self.auto_save.borrow() {
            self.save_to_settings();
        }
        self.scrolled_window.unparent();
    }
}

impl WidgetImpl for CustomTextView {}