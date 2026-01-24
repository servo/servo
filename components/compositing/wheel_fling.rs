/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Wheel fling support for platforms that don't provide native fling events (e.g., Linux).

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use base::id::WebViewId;
use embedder_traits::WheelPhase;
use euclid::Vector2D;
use webrender_api::units::{DevicePixel, DevicePoint, DeviceVector2D};

use crate::paint::RepaintReason;
use crate::painter::Painter;
use crate::refresh_driver::{BaseRefreshDriver, RefreshDriverObserver};
use crate::webview_renderer::WebViewRenderer;

/// Factor by which the flinging velocity changes on each tick.
const FLING_SCALING_FACTOR: f32 = 0.98;
/// Minimum velocity required to continue flinging (in device pixels per frame).
const FLING_MIN_VELOCITY: f32 = 1.0;
/// Maximum velocity when flinging (in device pixels per frame).
const FLING_MAX_VELOCITY: f32 = 4000.0;
/// Maximum time (in ms) between the last scroll movement and lifting fingers
/// for a fling to be triggered.
const FLING_START_TIMEOUT_MS: u128 = 100;

/// State of the wheel fling handler.
#[derive(Clone, Copy, Debug, PartialEq)]
enum WheelFlingState {
    /// Not currently tracking wheel events or flinging.
    Idle,
    /// Accumulating velocity from wheel scroll events.
    Scrolling {
        velocity: Vector2D<f32, DevicePixel>,
        last_point: DevicePoint,
    },
    /// Actively flinging with momentum.
    Flinging {
        velocity: Vector2D<f32, DevicePixel>,
        point: DevicePoint,
    },
}

/// An action that can be performed during wheel fling animation.
pub(crate) struct WheelFlingAction {
    pub delta: DeviceVector2D,
    pub cursor: DevicePoint,
}

/// Handler for wheel-based fling/momentum scrolling state.
pub(crate) struct WheelFlingHandler {
    webview_id: WebViewId,
    state: WheelFlingState,
    observing_frames_for_fling: bool,
    last_scroll_time: Option<Instant>,
}

impl WheelFlingHandler {
    pub(crate) fn new(webview_id: WebViewId) -> Self {
        Self {
            webview_id,
            state: WheelFlingState::Idle,
            observing_frames_for_fling: false,
            last_scroll_time: None,
        }
    }

    /// Called when a wheel event is received. Returns true if a fling should be started.
    pub(crate) fn on_wheel_event(
        &mut self,
        delta: DeviceVector2D,
        point: DevicePoint,
        phase: WheelPhase,
    ) -> bool {
        // If we're currently flinging and receive any wheel event, stop the fling first.
        // This stops the fling when the user starts a new scroll gesture. Note that on most
        // platforms, simply touching the trackpad without moving doesn't generate a wheel event,
        // so the fling will only stop once the user starts scrolling (receives Started/Moved).
        if self.is_flinging() {
            self.stop_fling();
        }

        match phase {
            WheelPhase::Started => {
                self.state = WheelFlingState::Scrolling {
                    velocity: Vector2D::new(delta.x, delta.y),
                    last_point: point,
                };
                self.last_scroll_time = Some(Instant::now());
            },
            WheelPhase::Moved => {
                self.last_scroll_time = Some(Instant::now());
                match self.state {
                    WheelFlingState::Idle => {
                        // Started event might have been missed, start tracking now.
                        self.state = WheelFlingState::Scrolling {
                            velocity: Vector2D::new(delta.x, delta.y),
                            last_point: point,
                        };
                    },
                    WheelFlingState::Scrolling {
                        ref mut velocity,
                        ref mut last_point,
                    } => {
                        // Accumulate velocity with light smoothing
                        // Use weighted average favoring the new delta for responsiveness.
                        *velocity = *velocity * 0.3 + Vector2D::new(delta.x, delta.y) * 0.7;
                        *last_point = point;
                    },
                    WheelFlingState::Flinging { .. } => {
                        // User started scrolling again without a Started event.
                        // Stop fling and start new scroll tracking
                        self.observing_frames_for_fling = false;
                        self.state = WheelFlingState::Scrolling {
                            velocity: Vector2D::new(delta.x, delta.y),
                            last_point: point,
                        };
                    },
                }
            },
            WheelPhase::Ended => {
                if let WheelFlingState::Scrolling {
                    velocity,
                    last_point,
                } = self.state
                {
                    let elapsed_since_last_scroll = self
                        .last_scroll_time
                        .map(|t| t.elapsed().as_millis())
                        .unwrap_or(u128::MAX);

                    // Don't start a fling if idle for too long before lifting fingers.
                    if elapsed_since_last_scroll > FLING_START_TIMEOUT_MS {
                        self.stop_fling();
                        return false;
                    }

                    if velocity.length().abs() >= FLING_MIN_VELOCITY {
                        let velocity = velocity.with_max_length(FLING_MAX_VELOCITY);
                        self.state = WheelFlingState::Flinging {
                            velocity,
                            point: last_point,
                        };
                        self.last_scroll_time = None;
                        return true;
                    }
                }
                // Velocity too low or not scrolling, just stop
                self.stop_fling();
            },
            WheelPhase::Cancelled => {
                self.stop_fling();
            },
        }
        false
    }

    /// Called at the start of each frame during fling animation.
    /// Returns the fling action to apply, or None if not currently flinging.
    pub(crate) fn notify_new_frame_start(&mut self) -> Option<WheelFlingAction> {
        let WheelFlingState::Flinging {
            ref mut velocity,
            point,
        } = self.state
        else {
            return None;
        };

        if velocity.length().abs() < FLING_MIN_VELOCITY {
            self.stop_fling();
            return None;
        }

        // Apply decay
        *velocity *= FLING_SCALING_FACTOR;

        Some(WheelFlingAction {
            delta: DeviceVector2D::new(velocity.x, velocity.y),
            cursor: point,
        })
    }

    /// Stop any ongoing fling.
    fn stop_fling(&mut self) {
        self.state = WheelFlingState::Idle;
        self.last_scroll_time = None;
        self.observing_frames_for_fling = false;
    }

    /// Returns true if currently in fling state.
    fn is_flinging(&self) -> bool {
        matches!(self.state, WheelFlingState::Flinging { .. })
    }

    /// Add a refresh driver observer to continue the fling animation if necessary.
    pub(crate) fn add_fling_refresh_observer_if_necessary(
        &mut self,
        refresh_driver: Rc<BaseRefreshDriver>,
        repaint_reason: &Cell<RepaintReason>,
    ) {
        if self.observing_frames_for_fling || !self.is_flinging() {
            return;
        }

        refresh_driver.add_observer(Rc::new(WheelFlingRefreshDriverObserver {
            webview_id: self.webview_id,
        }));
        self.observing_frames_for_fling = true;
        repaint_reason.set(repaint_reason.get().union(RepaintReason::StartedFlinging));
    }
}

/// Observer that drives the wheel fling animation on each frame.
pub(crate) struct WheelFlingRefreshDriverObserver {
    pub webview_id: WebViewId,
}

impl RefreshDriverObserver for WheelFlingRefreshDriverObserver {
    fn frame_started(&self, painter: &mut Painter) -> bool {
        painter
            .webview_renderer_mut(self.webview_id)
            .is_some_and(WebViewRenderer::update_wheel_fling_at_new_frame_start)
    }
}
