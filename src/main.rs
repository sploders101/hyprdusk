use std::path::Path;

use gio::{prelude::*, glib::SpawnFlags};
use gtk::{prelude::*, gdk::RGBA, StateFlags};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use zoha_vte::{traits::TerminalExt, PtyFlags};

fn activate(application: &gtk::Application, styles: &gtk::CssProvider) {
    // Create a normal GTK window however you like
    let window = gtk::ApplicationWindow::new(application);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();
    window.set_namespace("cava");

    // Display it above normal windows
    window.set_layer(Layer::Background);

    // Push other windows out of the way
    window.auto_exclusive_zone_enable();

    window.set_layer_shell_margin(Edge::Left, 20);
    window.set_layer_shell_margin(Edge::Right, 20);
    window.set_layer_shell_margin(Edge::Bottom, 0);

    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom,true);
    window.set_height_request(150);
    window.set_border_width(0);

    // Set up a terminal
    let terminal = zoha_vte::Terminal::new();
    terminal.set_color_background(&RGBA::new(0.0, 0.0, 0.0, 0.0));
    terminal.style_context().add_provider(styles, gtk::STYLE_PROVIDER_PRIORITY_USER);
    window.set_child(Some(&terminal));

    terminal.spawn_sync(
        PtyFlags::DEFAULT,
        None,
        &[Path::new("cava")],
        &[],
        SpawnFlags::DEFAULT,
        Some(&mut || {}),
        None::<&gio::Cancellable>,
    ).unwrap();

    application.connect_activate(move |_| {
        window.show_all();
    });

}

fn main() {
    let application = gtk::Application::new(Some("com.shaunkeys.hyprdusk.playground"), Default::default());

    application.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        // provider.load_from_path("./src/style.css").expect("Failed to load css");
        provider.load_from_data(include_bytes!("./style.css")).expect("Failed to load css");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        activate(app, &provider);
    });

    application.run();
}
