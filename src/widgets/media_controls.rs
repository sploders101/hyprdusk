use std::time::Duration;

use gtk::prelude::*;

use crate::services::Services;

pub fn media_control_button(services: &Services) -> gtk::Button {
    let button = gtk::Button::new();
    button.style_context().add_class("media-controller");
    let button_b = button.clone();

    let label = gtk::Label::new(Some("Test label"));
    button.add(&label);

    return button;
}
