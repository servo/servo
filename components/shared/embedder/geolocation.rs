/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types used to request geographic position data from the embedding layer.
//!
//! See <https://www.w3.org/TR/geolocation/>.

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::id::{PipelineId, WebViewId};

/// Identifies a single ongoing watchPosition() subscription. A subscription lives as long as
/// the Document that started it, so the identifier is scoped to a pipeline rather than being
/// globally unique.
#[derive(Clone, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct GeolocationWatchId {
    #[doc(hidden)]
    pub webview_id: WebViewId,
    #[doc(hidden)]
    pub pipeline_id: PipelineId,
    /// The `watchId` returned to script by `watchPosition()`.
    #[doc(hidden)]
    pub index: u32,
}

/// Hints about how a position should be acquired.
#[derive(Clone, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct GeolocationRequestOptions {
    /// A hint that the application would like to receive the most accurate location data
    /// available, at the cost of response time and power consumption.
    ///
    /// <https://www.w3.org/TR/geolocation/#dom-positionoptions-enablehighaccuracy>
    pub high_accuracy: bool,
}

/// A position acquired from the underlying system.
///
/// This mirrors the spec's "positionData" map, which is used to construct a
/// GeolocationCoordinates.
///
/// <https://www.w3.org/TR/geolocation/#dfn-acquire-a-position>
#[derive(Clone, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct GeolocationPositionData {
    /// The accuracy, in meters, at the 95% confidence level. Must not be negative.
    pub accuracy: f64,
    /// Degrees north of the Equator.
    pub latitude: f64,
    /// Degrees east of the Prime Meridian.
    pub longitude: f64,
    /// Meters above the WGS84 ellipsoid, or `None` if not available.
    pub altitude: Option<f64>,
    /// Accuracy of [`Self::altitude`] in meters at the 95% confidence level, or `None` if not
    /// available. Must not be negative.
    pub altitude_accuracy: Option<f64>,
    /// Direction of travel in degrees clockwise from true north, or `None` if not available.
    /// Servo reports `None` to script whenever [`Self::speed`] is zero, so an embedder does not
    /// need to special-case a stationary device.
    pub heading: Option<f64>,
    /// Ground speed in meters per second, or `None` if not available. Must not be negative.
    pub speed: Option<f64>,
}

impl GeolocationPositionData {
    /// Create position data for a device whose altitude, heading and speed are unknown.
    pub fn new(latitude: f64, longitude: f64, accuracy: f64) -> Self {
        Self {
            accuracy,
            latitude,
            longitude,
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        }
    }
}

/// The reason a position could not be acquired. These map directly onto the constants of
/// `GeolocationPositionError`.
///
/// <https://www.w3.org/TR/geolocation/#position_error_interface>
#[derive(Clone, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum GeolocationErrorKind {
    /// The user or the operating system refused access to the location service. Note that this is
    /// distinct from the user refusing Servo's own permission prompt, which never reaches the
    /// embedder's geolocation backend.
    PermissionDenied,
    /// The location service failed, or is not available on this platform.
    PositionUnavailable,
    /// Acquiring a position took longer than the page allowed.
    Timeout,
}

/// A failure to acquire a position, as reported by the embedding layer.
#[derive(Clone, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct GeolocationError {
    /// Which of the `GeolocationPositionError` constants to report to script.
    pub kind: GeolocationErrorKind,
    /// A human-readable description. The spec does not constrain this value.
    pub message: String,
}

impl GeolocationError {
    /// A [`GeolocationErrorKind::PositionUnavailable`] error with the given message.
    pub fn position_unavailable(message: impl Into<String>) -> Self {
        Self {
            kind: GeolocationErrorKind::PositionUnavailable,
            message: message.into(),
        }
    }

    /// A [`GeolocationErrorKind::PermissionDenied`] error with the given message.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: GeolocationErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    /// A [`GeolocationErrorKind::Timeout`] error with the given message.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self {
            kind: GeolocationErrorKind::Timeout,
            message: message.into(),
        }
    }
}

/// The reply sent back to script for a one-shot position request, or for each update of an
/// ongoing watch.
pub type GeolocationResult = Result<GeolocationPositionData, GeolocationError>;
