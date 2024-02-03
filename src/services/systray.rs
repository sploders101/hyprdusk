use std::{collections::HashMap, cmp::Ordering};

use system_tray::message::{tray::{IconPixmap, StatusNotifierItem}, menu::TrayMenu};
pub use system_tray::{NotifierItemMessage, message::NotifierItemCommand};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use gtk::prelude::*;


// This implementation is really gross. I don't think the protocol is actually that
// complicated, but this module adds a lot of unnecessary complexity. It's still
// easier than learning the protocol itself though, since I have no experience with
// dbus, so we'll go with this for now. I'll come back and rewrite this once I have
// something that works.

pub struct SystrayService {
    listener_channel: UnboundedSender<(tokio::sync::watch::Sender<HashMap<String, NotifierItem>>, UnboundedReceiver<NotifierItemCommand>)>,
}
impl Default for SystrayService {
    fn default() -> Self {
        let (channel_sender, mut channel_receiver) = mpsc::unbounded_channel::<(tokio::sync::watch::Sender<HashMap<String, NotifierItem>>, UnboundedReceiver<NotifierItemCommand>)>();

        tokio::task::spawn(async move {

            
            if let Some((client_sender, mut client_receiver)) = channel_receiver.recv().await {
                let (stray_sender, stray_receiver) = tokio::sync::mpsc::channel(32);
                let tray = system_tray::StatusNotifierWatcher::new(stray_receiver).await.unwrap();

                tokio::task::spawn(async move {
                    let mut host: Option<_> = None;
                    for _ in 0..10 {
                        if let Ok(new_host) = tray.create_notifier_host("Hyprdusk").await {
                            host = Some(new_host);
                            break;
                        }
                        if cfg!(debug_assertions) {
                            eprintln!("Failed to create notifier host. Trying again...");
                        }
                    }
                    let mut host = match host {
                        Some(host) => host,
                        None => return,
                    };
                    let mut state = HashMap::<String, NotifierItem>::new();
                    while let Ok(msg) = host.recv().await {
                        match msg {
                            NotifierItemMessage::Update {
                                address: id,
                                item,
                                menu,
                            } => {
                                state.insert(id, NotifierItem { item: *item, menu });
                            }
                            NotifierItemMessage::Remove { address } => {
                                state.remove(&address);
                            }
                        }
                        if let Err(_) = client_sender.send(state.clone()) {
                            break;
                        }
                    }
                });
                let stray_sender = stray_sender.clone();
                tokio::task::spawn(async move {
                    while let Some(msg) = client_receiver.recv().await {
                        if let Err(_) = stray_sender.send(msg).await {
                            break;
                        }
                    }
                });
            }
        });

        return Self {
            listener_channel: channel_sender,
        };
    }
}
impl SystrayService {
    pub fn create_facet(&self) -> SystrayFacet {
        let (gtk_sender, gtk_receiver) = tokio::sync::watch::channel(Default::default());
        let (tokio_sender, tokio_receiver) = tokio::sync::mpsc::unbounded_channel();
        let _ = self.listener_channel.send((gtk_sender, tokio_receiver));
        return SystrayFacet {
            sender: tokio_sender,
            receiver: Some(gtk_receiver),
        };
    }
}

pub struct SystrayFacet {
    sender: UnboundedSender<NotifierItemCommand>,
    receiver: Option<tokio::sync::watch::Receiver<HashMap<String, NotifierItem>>>,
}
impl Clone for SystrayFacet {
    fn clone(&self) -> Self {
        return SystrayFacet {
            sender: self.sender.clone(),
            receiver: None,
        };
    }
}
impl SystrayFacet {
    /// Attach a callback to the systray facet. This function must be called only once.
    pub fn attach(&mut self, mut callback: impl FnMut(&HashMap<String, NotifierItem>) -> glib::ControlFlow + 'static) {
        match self.receiver.take() {
            Some(mut receiver) => {
                glib::spawn_future_local(async move {
                    while let Ok(()) = receiver.changed().await {
                        let item = receiver.borrow();
                        callback(&item);
                    }
                });
            }
            None => {
                eprintln!("Multiple calls to `SystrayFacet::attach`. This is not allowed.");
            }
        }
    }
    pub fn send_command(&self, command: NotifierItemCommand) {
        let _ = self.sender.send(command);
    }
}



#[derive(Clone)]
pub struct NotifierItem {
    pub item: StatusNotifierItem,
    pub menu: Option<TrayMenu>,
}
impl NotifierItem {
    pub fn get_icon(&self) -> Option<gtk::Image> {
        match &self.item.icon_pixmap {
            None => self.get_icon_from_theme(),
            Some(pixmaps) => self.get_icon_from_pixmaps(pixmaps),
        }
    }

    pub fn get_icon_from_pixmaps(&self, pixmaps: &[IconPixmap]) -> Option<gtk::Image> {
        let mut pixmap = Vec::from(pixmaps);
        pixmap.sort_unstable_by(|pm1, pm2| {
            match (pm1.height, pm2.height) {
                (20..=24, 20..=24) => pm1.height.cmp(&pm2.height),
                (20..=24, _) => Ordering::Greater,
                (_, 20..=24) => Ordering::Less,
                (25.., 25..) => (pm1.height - 24).cmp(&(pm2.height - 24)),
                (25.., _) => Ordering::Less,
                (_, 25..) => Ordering::Less,
                (_, _) => pm1.height.cmp(&pm2.height),
            }
        });
        let pixmap = pixmap.pop().expect("No icon found");

        let mut pixbuf = gtk::gdk_pixbuf::Pixbuf::new(
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            pixmap.width,
            pixmap.height,
        )
        .expect("Failed to allocate pixbuf");

        for y in 0..pixmap.height {
            for x in 0..pixmap.width {
                let index = (y * pixmap.width + x) * 4;
                let a = pixmap.pixels[index as usize];
                let r = pixmap.pixels[(index + 1) as usize];
                let g = pixmap.pixels[(index + 2) as usize];
                let b = pixmap.pixels[(index + 3) as usize];
                pixbuf.put_pixel(x as u32, y as u32, r, g, b, a);
            }
        }

        if let Some(new_pixbuf) = pixbuf.scale_simple(24, 24, gdk::gdk_pixbuf::InterpType::Bilinear) {
            pixbuf = new_pixbuf;
        } else {
            eprintln!("Picture scaling failed");
        }

        let img = gtk::Image::from_pixbuf(Some(&pixbuf));

        img.set_height_request(24);

        None
        // Some(img)
    }

    pub fn get_icon_from_theme(&self) -> Option<gtk::Image> {
        let theme = gtk::IconTheme::default().unwrap_or(gtk::IconTheme::new());
        theme.rescan_if_needed();

        if let Some(path) = self.item.icon_theme_path.as_ref() {
            theme.append_search_path(path);
        }

        let icon_name = self.item.icon_name.as_ref().unwrap();
        let icon = theme.lookup_icon(icon_name, 24, gtk::IconLookupFlags::GENERIC_FALLBACK);

        icon.map(|i| {
            let pixbuf = i
                .load_icon()
                .ok()
                .and_then(|pixbuf| pixbuf.scale_simple(24, 24, gdk::gdk_pixbuf::InterpType::Bilinear));
            let img = gtk::Image::from_pixbuf(pixbuf.as_ref());

            img.set_height_request(32);

            img
        })
    }
}
