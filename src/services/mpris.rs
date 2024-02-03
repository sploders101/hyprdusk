use std::{sync::{Arc, Mutex}, collections::HashMap};

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, Receiver, Sender};

enum MprisEvent {
    /// Upon receiving this event, the button should disappear
    Error,
    Playing(NowPlaying),
    NotPlaying,
}

struct NowPlaying {
    app_id: String,
    app_title: String,
    app_icon: Option<String>,
    track_title: Option<String>,
    track_artist: Option<String>,
    track_album: Option<String>,
    track_playlist: Option<String>,
}

struct MprisController {
    player_finder: mpris::PlayerFinder,
    tracked_players: HashMap<String, Arc<mpris::Player>>,
    event_receiver: (Sender<()>, Receiver<()>),
}
impl MprisController {
    pub async fn scan(&mut self) -> Result<(), mpris::DBusError> {
        let player_iter = self.player_finder.iter_players()?;
        for player in player_iter {
            let player = Arc::new(player?);
            let name = player.unique_name();

            if !self.tracked_players.contains_key(name) {
                let sender = self.event_receiver.0.clone();
                self.tracked_players.insert(name.to_string(), Arc::clone(&player));
                tokio::task::spawn_blocking(move || {
                    let player = player;
                    let events = match player.events() {
                        Ok(events) => events,
                        Err(err) => {
                            eprintln!("{err:?}");
                            return;
                        }
                    };
                    for event in events {
                        sender.send(());
                    }
                });
            }
        }
        return Ok(());
    }
}


pub struct MprisService {
    listener_channel: UnboundedSender<glib::Sender<MprisEvent>>,
}
impl Default for MprisService {
    fn default() -> Self {
        let (sender, mut receiver) = unbounded_channel();
        let listeners = Arc::new(Mutex::new(Vec::<glib::Sender<MprisEvent>>::new()));
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
            let state = HashMap::<String, PlayerController>::new();
            match mpris::PlayerFinder::new() {
                Ok(player_finder) => {
                    match player_finder.iter_players() {
                        Ok(player_iter) => {
                            for player in player_iter {
                                match player {
                                    Ok(player) => {
                                        let name = player.unique_name();
                                        println!("Found player: {}", player.unique_name());
                                    }
                                    Err(err) => {
                                        eprintln!("{err:?}");
                                    }
                                }
                            }
                            println!("Exited loop");
                        }
                        Err(err) => {
                            eprintln!("{err:?}");
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{err:?}");
                    listeners.lock().unwrap().retain(|listener| listener.send(MprisEvent::Error).is_ok());
                }
            }
        });
        return MprisService {
            listener_channel: sender,
        };
    }
}
impl MprisService {
    // pub fn connect_window_change(&self, mut callback: impl FnMut(Option<WindowEventData>) -> glib::ControlFlow + 'static) {
    //     let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    //     if let Err(err) = self.listener_channel.send(sender) {
    //         eprintln!("{err:?}");
    //     }
    //     receiver.attach(None, move |event| if let HyprlandEvent::ActiveWindowChange(data) = event {
    //         callback(data)
    //     } else {
    //         glib::ControlFlow::Continue
    //     });
    // }
}
