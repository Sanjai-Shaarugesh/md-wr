use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use gtk::{gio, glib};

mod custom_textview;
use custom_textview::CustomTextView;

const APP_ID: &str = "org.md-wr.com";

fn main() {
    let resources_bytes = include_bytes!("../resources.gresource");
    let resource_data = glib::Bytes::from(&resources_bytes[..]);
    let resource = gio::Resource::from_data(&resource_data).expect("Failed to load resources");
    gio::resources_register(&resource);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(700)
            .build();

        // Create CustomTextView that fills the entire window
        let notes_textview = CustomTextView::with_settings_key("user-notes");
        notes_textview.set_placeholder_text("Start writing your masterpiece...");
        notes_textview.set_monospace(false);

        // Connect to text changes for notes
        notes_textview.connect_text_changed(|textview| {
            let text = textview.get_text();
            println!("Notes updated! Length: {}", text.len());
        });

        // Set the text view as the main window content
        window.set_content(Some(&notes_textview));

        window.present();
    });

    app.run();
}
