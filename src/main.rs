use std::path::Path;

use gio::{prelude::*, glib::SpawnFlags};
use gtk::{prelude::*, gdk::RGBA};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use zoha_vte::{traits::TerminalExt, PtyFlags};

fn activate(application: &gtk::Application) {
    // Create a normal GTK window however you like
    let window = gtk::ApplicationWindow::new(application);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();
    window.set_namespace("cava");

    // Display it above normal windows
    window.set_layer(Layer::Background);

    // Push other windows out of the way
    // window.auto_exclusive_zone_enable();

    window.set_layer_shell_margin(Edge::Left, 20);
    window.set_layer_shell_margin(Edge::Right, 20);
    window.set_layer_shell_margin(Edge::Bottom, 0);

    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, true);
    window.set_height_request(300);

    // Set up a terminal
    let terminal = zoha_vte::Terminal::new();
    terminal.set_color_background(&RGBA::new(0.0, 0.0, 0.0, 0.0));
    window.set_child(Some(&terminal));

    // Set up a widget
    // let label = gtk::Label::new(Some(""));
    // label.set_markup("<span font_desc=\"20.0\">GTK Layer Shell example!</span>");
    // window.add(&label);
    window.set_border_width(0);
    window.show_all();

    terminal.spawn_sync(
        PtyFlags::DEFAULT,
        None,
        &[Path::new("cava")],
        &[],
        SpawnFlags::DEFAULT,
        Some(&mut || {}),
        None::<&gio::Cancellable>,
    ).unwrap();
}

fn main() {
    let application = gtk::Application::new(Some("sh.wmww.gtk-layer-example"), Default::default());
    application.set_application_id(Some("testing"));

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}
