use std::sync::Arc;
use std::thread;

use geolocation_traits::{GeolocationError, GeolocationProvider, GeolocationRequest};
use ipc_channel::ipc;
use ipc_channel::ipc::{IpcReceiver, IpcSender};

pub struct GeolocationManager {
    provider: Arc<Option<Box<dyn GeolocationProvider + Send + Sync>>>,
    receiver: IpcReceiver<GeolocationRequest>,
}

impl GeolocationManager {
    pub fn new(
        provider: Arc<Option<Box<dyn GeolocationProvider + Send + Sync>>>,
        receiver: IpcReceiver<GeolocationRequest>,
    ) -> Self {
        Self { provider, receiver }
    }

    pub fn start(&self) {
        while let Ok(req) = self.receiver.recv() {
            if let Some(provider) = &*self.provider {
                match req {
                    GeolocationRequest::GetPosition(_, sender) => {
                        let result = provider.get_location();
                        let _ = sender.send(result);
                    },
                    GeolocationRequest::WatchPosition(_, sender) => {
                        let result = provider.get_location();
                        let _ = sender.send(result);
                    },
                }
            } else {
                match req {
                    GeolocationRequest::GetPosition(_, sender) => {
                        let _ = sender.send(Err(GeolocationError::PositionUnavailable));
                    },
                    GeolocationRequest::WatchPosition(_, sender) => {
                        let _ = sender.send(Err(GeolocationError::PositionUnavailable));
                    },
                }
            }
        }
    }
}

pub trait GeolocationThreadFactory {
    fn new(provider: Arc<Option<Box<dyn GeolocationProvider + Send + Sync>>>) -> Self;
}

impl GeolocationThreadFactory for IpcSender<GeolocationRequest> {
    fn new(
        provider: Arc<Option<Box<dyn GeolocationProvider + Send + Sync>>>,
    ) -> IpcSender<GeolocationRequest> {
        let (sender, receiver) = ipc::channel().unwrap();
        thread::Builder::new()
            .name("Geolocation".to_owned())
            .spawn(move || {
                GeolocationManager::new(provider, receiver).start();
            })
            .expect("Thread spawning failed");
        sender
    }
}
