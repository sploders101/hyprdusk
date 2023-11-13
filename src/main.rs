use std::collections::HashSet;

use gio::prelude::*;
use gtk::prelude::*;
use gtk_layer_shell::{Edge, Layer, LayerShell};
use widgets::workspaces::workspaces_widget;

mod widgets;

fn activate(application: &gtk::Application, monitor: gdk::Monitor) {
    // Set up the window
    let window = gtk::ApplicationWindow::new(application);
    window.style_context().add_class("hyprdusk-bar");

    window.init_layer_shell();
    window.set_monitor(&monitor);
    window.set_namespace("hyprdusk-bar");

    window.set_layer(Layer::Top);
    window.auto_exclusive_zone_enable();

    window.set_layer_shell_margin(Edge::Top, 0);
    window.set_layer_shell_margin(Edge::Left, 0);
    window.set_layer_shell_margin(Edge::Right, 0);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, false);

    window.set_height_request(45);

    // Create basic structure within window
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    window.set_child(Some(&bar));

    let left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    left.set_hexpand(true);
    bar.add(&left);

    let center = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    center.set_halign(gtk::Align::Center);
    center.set_hexpand(true);
    bar.add(&center);

    let right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    right.set_halign(gtk::Align::End);
    right.set_hexpand(true);
    bar.add(&right);

    // Add some components
    let workspaces = workspaces_widget(10);
    left.add(&workspaces);

    // Get ready for activation
    application.connect_activate(move |_| {
        window.show_all();
    });
}

fn main() {
    let application =
        gtk::Application::new(Some("com.shaunkeys.hyprdusk.main"), Default::default());

    application.connect_startup(|app| {
        eprintln!("Application startup");
        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(grass::include!("scss/main.scss").as_bytes())
            .expect("Failed to load css");
        let screen = gdk::Screen::default().expect("Error initializing gtk css provider.");
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Display on *all* monitors
        let mut activated_monitors = HashSet::new();
        let display = screen.display();
        eprintln!("Iterating over monitors...");
        for i in 0..display.n_monitors() {
            let monitor = display.monitor(i);
            if let Some(monitor) = monitor {
                eprintln!("Opening for monitor {i}");
                activated_monitors.insert(monitor.clone());
                activate(app, monitor);
            } else {
                eprintln!("Monitor not found");
            }
        }
    });

    eprintln!("Running application");

    let code = application.run();

    eprintln!("Run exited with {code:?}");
}
