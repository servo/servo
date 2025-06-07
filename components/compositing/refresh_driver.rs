/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::hash_map::Values;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use base::id::WebViewId;
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::{Sender, select};
use embedder_traits::EventLoopWaker;
use log::warn;
use timers::{BoxedTimerCallback, TimerEventRequest, TimerScheduler};

use crate::compositor::RepaintReason;
use crate::webview_renderer::WebViewRenderer;

const FRAME_DURATION: Duration = Duration::from_millis(1000 / 120);

/// The [`RefreshDriver`] is responsible for controlling updates to aall `WebView`s
/// onscreen presentation. Currently, it only manages controlling animation update
/// requests.
///
/// The implementation is very basic at the moment, only requesting new animation
/// frames at a constant time after a repaint.
pub(crate) struct RefreshDriver {
    /// The channel on which messages can be sent to the Constellation.
    pub(crate) constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// Whether or not we are currently animating via a timer.
    pub(crate) animating: Cell<bool>,

    /// Whether or not we are waiting for our frame timeout to trigger
    pub(crate) waiting_for_frame_timeout: Arc<AtomicBool>,

    /// A [`TimerThread`] which is used to schedule frame timeouts in the future.
    timer_thread: TimerThread,

    /// An [`EventLoopWaker`] to be used to wake up the embedder when it is
    /// time to paint a frame.
    event_loop_waker: Box<dyn EventLoopWaker>,
}

impl RefreshDriver {
    pub(crate) fn new(
        constellation_sender: Sender<EmbedderToConstellationMessage>,
        event_loop_waker: Box<dyn EventLoopWaker>,
    ) -> Self {
        Self {
            constellation_sender,
            animating: Default::default(),
            waiting_for_frame_timeout: Default::default(),
            timer_thread: Default::default(),
            event_loop_waker,
        }
    }

    fn timer_callback(&self) -> BoxedTimerCallback {
        let waiting_for_frame_timeout = self.waiting_for_frame_timeout.clone();
        let event_loop_waker = self.event_loop_waker.clone_box();
        Box::new(move || {
            waiting_for_frame_timeout.store(false, Ordering::Relaxed);
            event_loop_waker.wake();
        })
    }

    /// Notify the [`RefreshDriver`] that a paint is about to happen. This will trigger
    /// new animation frames for all active `WebView`s and schedule a new frame deadline.
    pub(crate) fn notify_will_paint(
        &self,
        webview_renderers: Values<'_, WebViewId, WebViewRenderer>,
    ) {
        // If we are still waiting for the frame to timeout this paint was caused for some
        // non-animation related reason and we should wait until the frame timeout to trigger
        // the next one.
        if self.waiting_for_frame_timeout.load(Ordering::Relaxed) {
            return;
        }

        // If any WebViews are animating ask them to paint again for another animation tick.
        let animating_webviews: Vec<_> = webview_renderers
            .filter_map(|webview_renderer| {
                if webview_renderer.animating() {
                    Some(webview_renderer.id)
                } else {
                    None
                }
            })
            .collect();

        // If nothing is animating any longer, update our state and exit early without requesting
        // any noew frames nor triggering a new animation deadline.
        if animating_webviews.is_empty() {
            self.animating.set(false);
            return;
        }

        if let Err(error) =
            self.constellation_sender
                .send(EmbedderToConstellationMessage::TickAnimation(
                    animating_webviews,
                ))
        {
            warn!("Sending tick to constellation failed ({error:?}).");
        }

        // Queue the next frame deadline.
        self.animating.set(true);
        self.waiting_for_frame_timeout
            .store(true, Ordering::Relaxed);
        self.timer_thread
            .queue_timer(FRAME_DURATION, self.timer_callback());
    }

    /// Notify the [`RefreshDriver`] that the animation state of a particular `WebView`
    /// via its associated [`WebViewRenderer`] has changed. In the case that a `WebView`
    /// has started animating, the [`RefreshDriver`] will request a new frame from it
    /// immediately, but only render that frame at the next frame deadline.
    pub(crate) fn notify_animation_state_changed(&self, webview_renderer: &WebViewRenderer) {
        if !webview_renderer.animating() {
            // If no other WebView is animating we will officially stop animated once the
            // next frame has been painted.
            return;
        }

        if let Err(error) =
            self.constellation_sender
                .send(EmbedderToConstellationMessage::TickAnimation(vec![
                    webview_renderer.id,
                ]))
        {
            warn!("Sending tick to constellation failed ({error:?}).");
        }

        if self.animating.get() {
            return;
        }

        self.animating.set(true);
        self.waiting_for_frame_timeout
            .store(true, Ordering::Relaxed);
        self.timer_thread
            .queue_timer(FRAME_DURATION, self.timer_callback());
    }

    /// Whether or not the renderer should trigger a message to the embedder to request a
    /// repaint. This might be false if: we are animating and the repaint reason is just
    /// for a new frame. In that case, the renderer should wait until the frame timeout to
    /// ask the embedder to repaint.
    pub(crate) fn wait_to_paint(&self, repaint_reason: RepaintReason) -> bool {
        if !self.animating.get() || repaint_reason != RepaintReason::NewWebRenderFrame {
            return false;
        }

        self.waiting_for_frame_timeout.load(Ordering::Relaxed)
    }
}

enum TimerThreadMessage {
    Request(TimerEventRequest),
    Quit,
}

/// A thread that manages a [`TimerScheduler`] running in the background of the
/// [`RefreshDriver`]. This is necessary because we need a reliable way of waking up the
/// embedder's main thread, which may just be sleeping until the `EventLoopWaker` asks it
/// to wake up.
///
/// It would be nice to integrate this somehow into the embedder thread, but it would
/// require both some communication with the embedder and for all embedders to be well
/// behave respecting wakeup timeouts -- a bit too much to ask at the moment.
struct TimerThread {
    sender: Sender<TimerThreadMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl Drop for TimerThread {
    fn drop(&mut self) {
        let _ = self.sender.send(TimerThreadMessage::Quit);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl Default for TimerThread {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<TimerThreadMessage>();
        let join_handle = thread::Builder::new()
            .name(String::from("CompositorTimerThread"))
            .spawn(move || {
                let mut scheduler = TimerScheduler::default();

                loop {
                    select! {
                        recv(receiver) -> message => {
                            match message {
                                Ok(TimerThreadMessage::Request(request)) => {
                                    scheduler.schedule_timer(request);
                                },
                                _ => return,
                            }
                        },
                        recv(scheduler.wait_channel()) -> _message => {
                            scheduler.dispatch_completed_timers();
                        },
                    };
                }
            })
            .expect("Could not create RefreshDriver timer thread.");

        Self {
            sender,
            join_handle: Some(join_handle),
        }
    }
}

impl TimerThread {
    fn queue_timer(&self, duration: Duration, callback: BoxedTimerCallback) {
        let _ = self
            .sender
            .send(TimerThreadMessage::Request(TimerEventRequest {
                callback,
                duration,
            }));
    }
}
