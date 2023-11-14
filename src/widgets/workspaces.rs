use gtk::prelude::*;
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::*;

pub fn workspaces_widget(num_workspaces: i32, services: &crate::services::Services) -> gtk::Box {
    let workspaces_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mut workspace_buttons = Vec::<(i32, gtk::Button)>::new();

    for i in 1..=num_workspaces {
        let workspace_button = gtk::Button::new();
        workspaces_box.add(&workspace_button);

        let label = gtk::Label::new(Some(i.to_string().as_str()));
        workspace_button.set_child(Some(&label));

        workspace_button.connect_clicked(move |_| {
            if let Err(err) = Dispatch::call(DispatchType::Workspace(
                WorkspaceIdentifierWithSpecial::Id(i),
            )) {
                eprintln!("{err:?}");
            }
        });

        workspace_buttons.push((i as i32, workspace_button));
    }

    let update_current_workspace = move || match hyprland::data::Workspace::get_active() {
        Ok(workspace) => {
            for (id, button) in workspace_buttons.iter() {
                if workspace.id == *id {
                    button.style_context().add_class("focused");
                } else {
                    button.style_context().remove_class("focused");
                }
            }
        }
        Err(err) => {
            eprintln!("{err:?}");
        }
    };
    update_current_workspace();

    services.hyprland.connect_workspace_change(move || {
        update_current_workspace();
        glib::ControlFlow::Continue
    });

    return workspaces_box;
}
