use std::sync::{Arc, Mutex};

use hyprland::event_listener::WindowEventData;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

enum HyprlandEvent {
    ActiveWindowChange(Option<WindowEventData>),
    ActiveWorkspaceChange,
}

pub struct HyprlandService {
    listener_channel: UnboundedSender<glib::Sender<HyprlandEvent>>,
}
impl Default for HyprlandService {
    fn default() -> Self {
        let (sender, mut receiver) = unbounded_channel();
        let listeners = Arc::new(Mutex::new(Vec::<glib::Sender<HyprlandEvent>>::new()));
        {
            let listeners = Arc::clone(&listeners);
            tokio::task::spawn(async move {
                loop {
                    if let Some(sender) = receiver.recv().await {
                        let mut listeners_ref = listeners.lock().unwrap();
                        listeners_ref.push(sender);
                        while let Ok(sender) = receiver.try_recv() {
                            listeners_ref.push(sender);
                        }
                    }
                }
            });
        }
        tokio::task::spawn(async move {
            let mut hypr_events = hyprland::event_listener::EventListener::new();

            {
                let listeners = Arc::clone(&listeners);
                hypr_events.add_active_window_change_handler(move |data| {
                    let mut listeners = listeners.lock().unwrap();
                    listeners.retain(|listener| listener.send(HyprlandEvent::ActiveWindowChange(data.clone())).is_ok());
                });
            }
            {
                let listeners = Arc::clone(&listeners);
                hypr_events.add_workspace_change_handler(move |_| {
                    let mut listeners = listeners.lock().unwrap();
                    listeners.retain(|listener| listener.send(HyprlandEvent::ActiveWorkspaceChange).is_ok());
                });
            }

            if let Err(err) = hypr_events.start_listener_async().await {
                eprintln!("{err:?}");
            }
        });
        return HyprlandService {
            listener_channel: sender,
        };
    }
}
impl HyprlandService {
    pub fn connect_window_change(&self, mut callback: impl FnMut(Option<WindowEventData>) -> glib::ControlFlow + 'static) {
        let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
        if let Err(err) = self.listener_channel.send(sender) {
            eprintln!("{err:?}");
        }
        receiver.attach(None, move |event| if let HyprlandEvent::ActiveWindowChange(data) = event {
            callback(data)
        } else {
            glib::ControlFlow::Continue
        });
    }
    pub fn connect_workspace_change(&self, mut callback: impl FnMut() -> glib::ControlFlow + 'static) {
        let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
        if let Err(err) = self.listener_channel.send(sender) {
            eprintln!("{err:?}");
        }
        receiver.attach(None, move |event| if let HyprlandEvent::ActiveWorkspaceChange = event {
            callback()
        } else {
            glib::ControlFlow::Continue
        });
    }
}
