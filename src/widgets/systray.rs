use gtk::prelude::*;
use system_tray::message::{menu::MenuType, NotifierItemCommand};
use crate::services::{self, systray::SystrayFacet};


pub struct StatusNotifierWrapper {
    menu: system_tray::message::menu::MenuItem,
}
impl StatusNotifierWrapper {
    fn into_menu_item(
        self,
        sender: &SystrayFacet,
        notifier_address: String,
        menu_path: String,
    ) -> gtk::MenuItem {
        let item: Box<dyn AsRef<gtk::MenuItem>> = match self.menu.menu_type {
            MenuType::Separator => Box::new(gtk::SeparatorMenuItem::new()),
            MenuType::Standard => Box::new(gtk::MenuItem::with_label(self.menu.label.as_str())),
        };

        let item = (*item).as_ref().clone();

        {
            let sender = sender.clone();
            let notifier_address = notifier_address.clone();
            let menu_path = menu_path.clone();

            item.connect_activate(move |_item| {
                sender.send_command(NotifierItemCommand::MenuItemClicked {
                    submenu_id: self.menu.id,
                    menu_path: menu_path.clone(),
                    notifier_address: notifier_address.clone(),
                });
            });
        };

        let submenu = gtk::Menu::new();
        if !self.menu.submenu.is_empty() {
            for submenu_item in self.menu.submenu.iter().cloned() {
                let submenu_item = StatusNotifierWrapper { menu: submenu_item };
                let submenu_item = submenu_item.into_menu_item(
                    sender,
                    notifier_address.clone(),
                    menu_path.clone(),
                );
                submenu.append(&submenu_item);
            }

            item.set_submenu(Some(&submenu));
        }

        item
    }
}

pub fn system_tray(services: &services::Services) -> gtk::MenuBar {
    let menu = gtk::MenuBar::new();
    menu.style_context().add_class("systray");
    let mut systray_facet_recv = services.systray.create_facet();

    {
        let menu = menu.clone();
        let systray_facet = systray_facet_recv.clone();
        systray_facet_recv.attach(move |state| {
            for child in menu.children() {
                menu.remove(&child);
            }

            for (address, notifier_item) in state.iter() {
                if let Some(icon) = notifier_item.get_icon() {
                    // Create the menu

                    let menu_item = gtk::MenuItem::new();
                    let menu_item_box = gtk::Box::default();
                    menu_item_box.add(&icon);
                    menu_item.add(&menu_item_box);

                    if let Some(tray_menu) = &notifier_item.menu {
                        let menu = gtk::Menu::new();
                        tray_menu
                            .submenus
                            .iter()
                            .map(|submenu| StatusNotifierWrapper {
                                menu: submenu.to_owned(),
                            })
                            .map(|item| {
                                let menu_path =
                                    notifier_item.item.menu.as_ref().unwrap().to_string();
                                let address = address.to_string();
                                item.into_menu_item(&systray_facet, address, menu_path)
                            })
                            .for_each(|item| menu.append(&item));

                        if !tray_menu.submenus.is_empty() {
                            menu_item.set_submenu(Some(&menu));
                        }
                    }
                    menu.append(&menu_item);
                };

                menu.show_all();
            }

            glib::ControlFlow::Continue
        });
    }

    return menu;
}
