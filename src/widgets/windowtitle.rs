use gtk::prelude::*;
use hyprland::prelude::*;
use crate::services;

pub fn window_title(services: &services::Services) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.style_context().add_class("window-title");

    let label_value_initial = match hyprland::data::Client::get_active() {
        Ok(Some(client)) => {
           Some(client.title) 
        }
        Ok(None) => {
            println!("No active client");
            None
        }
        Err(err) => {
            eprintln!("Error while getting active client: {err:?}");
            None
        }
    };
    let label = gtk::Label::new(label_value_initial.as_ref().map(|title| title.as_str()));
    container.add(&label);

    services.hyprland.connect_window_change(move |data| {
        if let Some(data) = data {
            label.set_text(data.window_title.as_str());
        }
        glib::ControlFlow::Continue
    });

    return container;
}
