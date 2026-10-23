/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`GeolocationDelegate`](crate::GeolocationDelegate) backend built on GeoClue.
//!
//! Details about the design:
//! - This runs as a task on the shared async runtime instead of using a dedicated thread.
//! - `Manager.GetClient()` returns one client per D-Bus connection, so every page using
//!   geolocation in this Servo instance shares a single client. The task multiplexes and
//!   keeps one client alive for as long as anything wants a position, requests the highest
//!   accuracy asked for, and dispatches each update to all consumers.
//! - GeoClue has no one-shot request. `getCurrentPosition()` is served by joining the shared
//!   client and taking the first position emitted by the stream.

use std::collections::HashMap;

use embedder_traits::{GeolocationError, GeolocationPositionData, GeolocationWatchId};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::{FutureExt, StreamExt, select};
use log::error;
use net::async_runtime;
use zbus::proxy::CacheProperties;
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, proxy};

use crate::geolocation_delegate::{PositionRequest, PositionWatch};

/// <https://www.freedesktop.org/software/geoclue/docs/geoclue-gclue-enums.html>
#[derive(Clone, Copy, PartialEq)]
#[repr(u32)]
enum AccuracyLevel {
    City = 4,
    Exact = 8,
}

/// GeoClue reports "no altitude" as a very large negative number rather than an absent value.
/// Firefox uses the same cutoff.
const MINIMUM_PLAUSIBLE_ALTITUDE: f64 = -500.0;

#[proxy(
    interface = "org.freedesktop.GeoClue2.Manager",
    default_service = "org.freedesktop.GeoClue2",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait Manager {
    fn get_client(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.GeoClue2.Client",
    default_service = "org.freedesktop.GeoClue2",
    assume_defaults = false
)]
trait Client {
    fn start(&self) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;

    /// The name of the application's desktop file, which GeoClue uses to grant some access.
    #[zbus(property)]
    fn set_desktop_id(&self, id: &str) -> zbus::Result<()>;

    #[zbus(property)]
    fn set_requested_accuracy_level(&self, level: u32) -> zbus::Result<()>;

    /// The most recent location, or `/` before there has been one.
    #[zbus(property)]
    fn location(&self) -> zbus::Result<OwnedObjectPath>;

    #[zbus(signal)]
    fn location_updated(
        &self,
        previous: OwnedObjectPath,
        current: OwnedObjectPath,
    ) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.GeoClue2.Location",
    default_service = "org.freedesktop.GeoClue2",
    assume_defaults = false
)]
trait Location {
    #[zbus(property)]
    fn latitude(&self) -> zbus::Result<f64>;
    #[zbus(property)]
    fn longitude(&self) -> zbus::Result<f64>;
    /// Horizontal accuracy in metres.
    #[zbus(property)]
    fn accuracy(&self) -> zbus::Result<f64>;
    #[zbus(property)]
    fn altitude(&self) -> zbus::Result<f64>;
    /// Metres per second, or negative if unknown.
    #[zbus(property)]
    fn speed(&self) -> zbus::Result<f64>;
    /// Degrees clockwise from true north, or negative if unknown.
    #[zbus(property)]
    fn heading(&self) -> zbus::Result<f64>;
}

/// Translate a D-Bus failure into the closest `GeolocationPositionError` code, matching the
/// mapping Firefox uses in `GeoclueLocationProvider::DBusProxyError`.
fn convert_error(error: zbus::Error) -> GeolocationError {
    let zbus::Error::MethodError(ref name, _, _) = error else {
        return GeolocationError::position_unavailable(error.to_string());
    };
    match name.as_str() {
        "org.freedesktop.DBus.Error.Timeout" | "org.freedesktop.DBus.Error.TimedOut" => {
            GeolocationError::timeout(error.to_string())
        },
        "org.freedesktop.DBus.Error.AccessDenied" |
        "org.freedesktop.DBus.Error.AuthFailed" |
        "org.freedesktop.DBus.Error.LimitsExceeded" => {
            GeolocationError::permission_denied(error.to_string())
        },
        _ => GeolocationError::position_unavailable(error.to_string()),
    }
}

async fn read_position(
    connection: &Connection,
    path: OwnedObjectPath,
) -> zbus::Result<GeolocationPositionData> {
    // A Location object is an immutable snapshot that GeoClue discards again, so caching its
    // properties would only add a PropertiesChanged subscription with nothing to report.
    let location = LocationProxy::builder(connection)
        .path(path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let altitude = location.altitude().await?;
    let speed = location.speed().await?;
    let heading = location.heading().await?;

    Ok(GeolocationPositionData {
        accuracy: location.accuracy().await?,
        latitude: location.latitude().await?,
        longitude: location.longitude().await?,
        altitude: (altitude >= MINIMUM_PLAUSIBLE_ALTITUDE).then_some(altitude),
        // GeoClue does not report a vertical accuracy.
        altitude_accuracy: None,
        heading: (heading >= 0.).then_some(heading),
        speed: (speed >= 0.).then_some(speed),
    })
}

/// Work handed to the GeoClue task by the delegate.
enum Request {
    /// A one-shot `getCurrentPosition()`, answered by the next position to arrive.
    Acquire(PositionRequest),
    StartWatch(PositionWatch),
    StopWatch(GeolocationWatchId),
}

/// A handle to the task that owns Servo's connection to GeoClue.
pub(crate) struct Geoclue(UnboundedSender<Request>);

impl Geoclue {
    /// Start the GeoClue task on Servo's shared runtime. The connection is created lazily, so a session
    /// that never uses geolocation pays nothing.
    pub(crate) fn spawn() -> Option<Self> {
        if !async_runtime::async_runtime_initialized() {
            error!("No async runtime yet; geolocation is unavailable.");
            return None;
        }
        let (sender, receiver) = unbounded();
        async_runtime::spawn_task(GeoclueTaskState::default().run(receiver));
        Some(Self(sender))
    }

    pub(crate) fn request_position(&self, request: PositionRequest) {
        let _ = self.0.unbounded_send(Request::Acquire(request));
    }

    pub(crate) fn start_watch(&self, watch: PositionWatch) {
        if self.0.unbounded_send(Request::StartWatch(watch)).is_err() {
            error!("The GeoClue task is gone; no positions will be delivered.");
        }
    }

    pub(crate) fn stop_watch(&self, id: &GeolocationWatchId) {
        let _ = self.0.unbounded_send(Request::StopWatch(id.clone()));
    }
}

/// The live connection to GeoClue, held only while something wants a position.
struct Session {
    connection: Connection,
    client: ClientProxy<'static>,
    accuracy: AccuracyLevel,
}

/// The state of the GeoClue task.
#[derive(Default)]
struct GeoclueTaskState {
    session: Option<Session>,
    /// Bumped whenever `session` is replaced or dropped, so that the run loop knows when its
    /// subscription belongs to a client that no longer exists.
    generation: u64,
    /// `getCurrentPosition()` calls waiting for the next position.
    pending: Vec<PositionRequest>,
    /// `watchPosition()` calls, for as long as script wants them.
    watches: HashMap<GeolocationWatchId, PositionWatch>,
}

impl GeoclueTaskState {
    async fn run(mut self, mut requests: UnboundedReceiver<Request>) {
        let mut updates_stream: Option<LocationUpdatedStream> = None;
        let mut subscribed_to = None;

        loop {
            if subscribed_to != Some(self.generation) {
                // Drop any subscription belonging to the previous dbus client before taking a new one.
                updates_stream = None;
                let client = self.session.as_ref().map(|session| session.client.clone());
                if let Some(client) = client {
                    match client.receive_location_updated().await {
                        Ok(stream) => updates_stream = Some(stream),
                        Err(error) => {
                            self.handle_failure(error);
                            continue;
                        },
                    }
                }
                subscribed_to = Some(self.generation);

                // Now that updates are being listened for, pick up a fix that may have landed
                // between starting the client and subscribing. Without this a device that never
                // moves again would never produce a new position.
                if updates_stream.is_some() && !self.is_idle() {
                    self.report_current_position().await;
                }
            }

            let next_update = async {
                match updates_stream.as_mut() {
                    Some(stream) => stream.next().await,
                    // Without a session there is no update to wait for, only requests.
                    None => futures::future::pending::<Option<LocationUpdated>>().await,
                }
            };

            select! {
                request = requests.next() => {
                    let Some(request) = request else {
                        // The delegate, and with it `Servo`, is gone.
                        break;
                    };
                    self.handle_request(request).await;
                },
                update = next_update.fuse() => match update {
                    Some(update) => self.handle_update(update).await,
                    // GeoClue went away.
                    None => self.close_session().await,
                },
            }
        }
        self.close_session().await;
    }

    async fn handle_request(&mut self, request: Request) {
        match request {
            Request::Acquire(request) => {
                let wants_high_accuracy = request.options().high_accuracy;
                self.pending.push(request);
                if let Err(error) = self.open_or_update_session(wants_high_accuracy).await {
                    self.handle_failure(error);
                    return;
                }
                // A position may already be available, in which case there is no reason to
                // wait for the device to move.
                self.report_current_position().await;
            },
            Request::StartWatch(watch) => {
                let wants_high_accuracy = watch.options().high_accuracy;
                self.watches.insert(watch.id(), watch);
                if let Err(error) = self.open_or_update_session(wants_high_accuracy).await {
                    self.handle_failure(error);
                    return;
                }
                self.report_current_position().await;
            },
            Request::StopWatch(id) => {
                self.watches.remove(&id);
                if self.is_idle() {
                    self.close_session().await;
                }
            },
        }
    }

    async fn handle_update(&mut self, update: LocationUpdated) {
        let Ok(args) = update.args() else {
            return;
        };
        let Some(connection) = self
            .session
            .as_ref()
            .map(|session| session.connection.clone())
        else {
            return;
        };
        match read_position(&connection, args.current().clone()).await {
            Ok(position) => self.report(Ok(position)).await,
            Err(error) => self.handle_failure(error),
        }
    }

    /// Deliver the position the client is already holding, if it has one. GeoClue points the
    /// client's `Location` property at `/` until the first fix.
    async fn report_current_position(&mut self) {
        let Some((connection, client)) = self
            .session
            .as_ref()
            .map(|session| (session.connection.clone(), session.client.clone()))
        else {
            error!("No client!");
            return;
        };
        let path = match client.location().await {
            Ok(path) if path.as_str() != "/" => path,
            Ok(_) => return,
            Err(error) => return self.handle_failure(error),
        };
        match read_position(&connection, path).await {
            Ok(position) => self.report(Ok(position)).await,
            Err(error) => self.handle_failure(error),
        }
    }

    /// Hand a result to everyone waiting for a position.
    ///
    /// A consumer joining a client that already has a fix makes every watch see that position
    /// again. Firefox re-delivers its last position to new watchers for the same reason.
    async fn report(&mut self, result: Result<GeolocationPositionData, GeolocationError>) {
        for request in self.pending.drain(..) {
            match result.clone() {
                Ok(position) => request.success(position),
                Err(error) => request.failure(error),
            }
        }
        for watch in self.watches.values() {
            match result.clone() {
                Ok(position) => watch.update(position),
                Err(error) => watch.error(error),
            }
        }
        if self.is_idle() {
            self.close_session().await;
        }
    }

    /// Report a D-Bus failure to script and tear the session down.
    ///
    /// If GeoClue denied us because its agent had not connected yet, `GetClient()` keeps
    /// returning the same rejected client, so the connection itself has to go for a later attempt
    /// to have any chance. Firefox carries the same workaround for
    /// <https://gitlab.freedesktop.org/geoclue/geoclue/-/issues/143>.
    fn handle_failure(&mut self, error: zbus::Error) {
        let error = convert_error(error);
        error!("GeoClue request failed: {}", error.message);

        // Dropped rather than stopped: after a denial the client is unusable, and after any other
        // failure the connection is suspect.
        self.session = None;

        for request in self.pending.drain(..) {
            request.failure(error.clone());
        }
        // Tell the watches too, rather than leaving them waiting on a session that no longer
        // exists. They stay registered, and the next request from script rebuilds the session and
        // starts feeding them again.
        for watch in self.watches.values() {
            watch.error(error.clone());
        }
    }

    fn is_idle(&self) -> bool {
        self.pending.is_empty() && self.watches.is_empty()
    }

    /// The accuracy to request: the most any current consumer asked for.
    fn wanted_accuracy(&self, wants_high_accuracy: bool) -> AccuracyLevel {
        let high = wants_high_accuracy ||
            self.pending
                .iter()
                .any(|request| request.options().high_accuracy) ||
            self.watches
                .values()
                .any(|watch| watch.options().high_accuracy);
        if high {
            AccuracyLevel::Exact
        } else {
            AccuracyLevel::City
        }
    }

    /// Make sure a started client exists, restarting it if the accuracy it was started with is no
    /// longer the one we want since GeoClue only honours `RequestedAccuracyLevel` at `Start()`.
    async fn open_or_update_session(&mut self, wants_high_accuracy: bool) -> zbus::Result<()> {
        let accuracy = self.wanted_accuracy(wants_high_accuracy);

        let existing = self
            .session
            .as_ref()
            .map(|session| (session.client.clone(), session.accuracy));
        if let Some((client, started_with)) = existing {
            if started_with == accuracy {
                return Ok(());
            }
            client.stop().await?;
            client.set_requested_accuracy_level(accuracy as u32).await?;
            client.start().await?;
            if let Some(session) = self.session.as_mut() {
                session.accuracy = accuracy;
            }
            return Ok(());
        }

        let connection = Connection::system().await?;
        let manager = ManagerProxy::new(&connection).await?;
        let client = ClientProxy::builder(&connection)
            .path(manager.get_client().await?)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        // Use "firefox" as our desktop ID since it's part of the default GeoClue
        // configuration on most Linux distributions.
        client.set_desktop_id("firefox").await?;

        client.set_requested_accuracy_level(accuracy as u32).await?;
        client.start().await?;

        self.session = Some(Session {
            connection,
            client,
            accuracy,
        });
        self.generation += 1;
        Ok(())
    }

    async fn close_session(&mut self) {
        let Some(session) = self.session.take() else {
            return;
        };
        if let Err(error) = session.client.stop().await {
            error!("Could not stop the GeoClue client ({error}).");
        }
    }
}
