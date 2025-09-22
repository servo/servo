use std::time::SystemTime;

use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Options {
    pub accuracy: Accuracy,
    pub maximum_age: u32,
    pub timeout: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Position {
    pub coords: Coordinates,
    pub timestamp: SystemTime,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Accuracy {
    High,
    Low,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeolocationError {
    /// The user denied permission to access their location.
    PermissionDenied,
    PositionUnavailable,
    Timeout,
    /// An unknown error occurred.
    Other(String),
}

pub trait GeolocationProvider {
    /// Sets the watcher callback that should be called when the location changes or an error occurs.
    /// The watcher can be set to None to stop receiving updates.
    /// The watcher can be set at any time, even after start() has been called.
    /// The watcher can also be set multiple times, and the previous watcher will be replaced,
    /// it is up to the provider to use interior mutability to ensure this works.
    fn set_watcher(&self, callback: Option<Box<dyn Fn(Result<Position, GeolocationError>) + Send>>);
    /// Starts the geolocation provider.
    /// The provider should stream location updates to the watcher callback if set.
    /// This method should not block, and should return immediately.
    fn start(&self);
    /// Stops the geolocation provider.
    /// The provider should stop streaming location updates to the watcher callback.
    /// The watcher callback should not be called after this method is called.
    /// This method should not block, and should return immediately.
    fn stop(&self);

    /// Returns the current location if available.
    /// If an error occurs while trying to get the location, returns a GeolocationError.
    /// This can block, as the script thread won't be blocked while waiting for a location update.
    fn get_location(&self) -> Result<Position, GeolocationError>;

    /// Sets the desired accuracy for location updates.
    /// This method can be called at any time to change the accuracy.
    /// However, the provider may choose to ignore this request, either for privacy reasons,
    /// or because the provider cannot honor the request due to technical limitations.
    fn set_accuracy(&self, accuracy: Accuracy);
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GeolocationRequest {
    /// One-shot request to get the current position.
    GetPosition(Options, IpcSender<Result<Position, GeolocationError>>),
    /// Continuous request to watch the position.
    /// Dropping the sender can be used to stop watching the position.
    WatchPosition(Options, IpcSender<Result<Position, GeolocationError>>),
}
