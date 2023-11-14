use tokio::runtime::Runtime;

pub mod hyprland;
pub mod systray;


#[derive(Default)]
pub struct Services {
    pub hyprland: self::hyprland::HyprlandService,
    pub systray: self::systray::SystrayService,
}
impl Services {
    /// Sets up a tokio runtime and spawns services' default implementations
    pub fn new() -> (Self, tokio::sync::oneshot::Sender<()>) {
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
        let (services_sender, services_receiver) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to spawn Tokio runtime");
            runtime.block_on(async {
                if let Err(_) = services_sender.send(Self::default()) {
                    panic!("Failed to send services");
                }
                let _ = shutdown_receiver.await;
            });
        });
        return (services_receiver.blocking_recv().expect("Failed to receive services"), shutdown_sender);
    }
}
