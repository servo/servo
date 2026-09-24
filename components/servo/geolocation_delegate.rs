/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{
    GeolocationError, GeolocationPositionData, GeolocationRequestOptions, GeolocationResult,
    GeolocationWatchId,
};
use servo_base::generic_channel::GenericCallback;
use servo_config::pref;

use crate::WebView;

/// A request for a single geolocation fix, corresponding to one call to
/// navigator.geolocation.getCurrentPosition()
pub struct PositionRequest {
    result_sender: GenericCallback<GeolocationResult>,
    options: GeolocationRequestOptions,
    response_sent: bool,
}

impl PositionRequest {
    pub(crate) fn new(
        result_sender: GenericCallback<GeolocationResult>,
        options: GeolocationRequestOptions,
    ) -> Self {
        Self {
            result_sender,
            options,
            response_sent: false,
        }
    }

    /// Hints about how the position should be acquired.
    pub fn options(&self) -> &GeolocationRequestOptions {
        &self.options
    }

    /// Report the acquired position.
    pub fn success(mut self, position: GeolocationPositionData) {
        let _ = self.result_sender.send(Ok(position));
        self.response_sent = true;
    }

    /// Report that the position could not be acquired.
    pub fn failure(mut self, error: GeolocationError) {
        let _ = self.result_sender.send(Err(error));
        self.response_sent = true;
    }
}

impl Drop for PositionRequest {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self
                .result_sender
                .send(Err(GeolocationError::position_unavailable(
                    "No response sent to request.",
                )));
        }
    }
}

/// An ongoing subscription to position updates, corresponding to one call to
/// navigator.geolocation.watchPosition()
pub struct PositionWatch {
    id: GeolocationWatchId,
    result_sender: GenericCallback<GeolocationResult>,
    options: GeolocationRequestOptions,
}

impl PositionWatch {
    pub(crate) fn new(
        id: GeolocationWatchId,
        result_sender: GenericCallback<GeolocationResult>,
        options: GeolocationRequestOptions,
    ) -> Self {
        Self {
            id,
            result_sender,
            options,
        }
    }

    /// The identifier used to stop this watch.
    pub fn id(&self) -> GeolocationWatchId {
        self.id.clone()
    }

    /// Hints about how positions should be acquired.
    pub fn options(&self) -> &GeolocationRequestOptions {
        &self.options
    }

    /// Deliver a new position to script. May be called any number of times.
    pub fn update(&self, position: GeolocationPositionData) {
        let _ = self.result_sender.send(Ok(position));
    }

    /// Report that a position could not be acquired. This does not end the watch: script keeps
    /// listening, and the embedder may still call PositionWatch::update() later.
    pub fn error(&self, error: GeolocationError) {
        let _ = self.result_sender.send(Err(error));
    }
}

/// A delegate responsible for acquiring the device's geographic position.
pub trait GeolocationDelegate {
    /// A request for a single position fix.
    fn request_position(&self, _webview: WebView, request: PositionRequest) {
        match default_position_response() {
            Ok(position) => request.success(position),
            Err(error) => request.failure(error),
        }
    }

    /// A request to start delivering continuous position updates.
    fn start_watch(&self, _webview: WebView, watch: PositionWatch) {
        match default_position_response() {
            Ok(position) => watch.update(position),
            Err(error) => watch.error(error),
        }
    }

    /// A request to stop delivering updates for a watch previously passed to
    /// GeolocationDelegate::start_watch(), and to drop the corresponding PositionWatch.
    fn stop_watch(&self, _webview: WebView, _id: &GeolocationWatchId) {}
}

/// The position described by the `dom_geolocation_test_position_*` preferences, or an error if
/// they are not set.
fn default_position_response() -> GeolocationResult {
    if !pref!(dom_geolocation_test_position_enabled) {
        return Err(GeolocationError::position_unavailable(
            "No geolocation backend is available.",
        ));
    }

    Ok(GeolocationPositionData {
        accuracy: pref!(dom_geolocation_test_accuracy),
        latitude: pref!(dom_geolocation_test_latitude),
        longitude: pref!(dom_geolocation_test_longitude),
        altitude: None,
        altitude_accuracy: None,
        heading: Some(pref!(dom_geolocation_test_heading)),
        speed: Some(pref!(dom_geolocation_test_speed)),
    })
}

pub(crate) struct DefaultGeolocationDelegate;

impl GeolocationDelegate for DefaultGeolocationDelegate {}
