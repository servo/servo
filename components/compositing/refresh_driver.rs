/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::{Sender, select};
use embedder_traits::{EventLoopWaker, RefreshDriver};
use log::warn;
use timers::{BoxedTimerCallback, TimerEventRequest, TimerScheduler};

use crate::compositor::RepaintReason;
use crate::painter::Painter;
use crate::webview_renderer::WebViewRenderer;

/// The [`BaseRefreshDriver`] is a "base class" for [`RefreshDriver`] trait
/// implementations. It encapsulates shared behavior so that it does not have to be
/// implemented by all trait implementations. It is responsible for providing
/// [`RefreshDriver`] implementations with a callback that is used to wake up the event
/// loop and trigger frame readiness.
pub(crate) struct BaseRefreshDriver {
    /// Whether or not the [`BaseRefreshDriver`] is waiting for a frame. Once the [`RefreshDriver`]
    /// informs the base that a frame start happened, this becomes false.
    waiting_for_frame: Arc<AtomicBool>,
    /// An [`EventLooperWaker`] which alerts the main UI event loop when a frame start occurs.
    event_loop_waker: Box<dyn EventLoopWaker>,
    /// A list of internal observers that watch for frame starts.
    observers: RefCell<Vec<Rc<dyn RefreshDriverObserver>>>,
    /// The implementation of the [`RefreshDriver`]. By default this is a simple timer, but the
    /// embedder can install a custom driver, such as one that is run via the hardware vsync signal.
    refresh_driver: Rc<dyn RefreshDriver>,
}

impl BaseRefreshDriver {
    pub(crate) fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        refresh_driver: Option<Rc<dyn RefreshDriver>>,
    ) -> Self {
        let refresh_driver =
            refresh_driver.unwrap_or_else(|| Rc::new(TimerRefreshDriver::default()));
        Self {
            waiting_for_frame: Arc::new(AtomicBool::new(false)),
            event_loop_waker,
            observers: Default::default(),
            refresh_driver,
        }
    }

    pub(crate) fn add_observer(&self, observer: Rc<dyn RefreshDriverObserver>) {
        let mut observers = self.observers.borrow_mut();
        observers.push(observer);

        // If this is the first observer, make sure to observe the next frame.
        if observers.len() == 1 {
            self.observe_next_frame();
        }
    }

    pub(crate) fn notify_will_paint(&self, renderer: &mut Painter) {
        // If we are still waiting for the frame to timeout this paint was caused for some
        // non-animation related reason and we should wait until the frame timeout to trigger
        // the next one.
        if self.waiting_for_frame.load(Ordering::Relaxed) {
            return;
        }

        // Limit the borrow of `self.observers` to the minimum here.
        let still_has_observers = {
            let mut observers = self.observers.borrow_mut();
            observers.retain(|observer| observer.frame_started(renderer));
            !observers.is_empty()
        };

        if still_has_observers {
            self.observe_next_frame();
        }
    }

    fn observe_next_frame(&self) {
        self.waiting_for_frame.store(true, Ordering::Relaxed);

        let waiting_for_frame = self.waiting_for_frame.clone();
        let event_loop_waker = self.event_loop_waker.clone_box();
        self.refresh_driver.observe_next_frame(Box::new(move || {
            waiting_for_frame.store(false, Ordering::Relaxed);
            event_loop_waker.wake();
        }));
    }

    /// Whether or not the renderer should trigger a message to the embedder to request a
    /// repaint. This might be true if we are animating and the repaint reason is just
    /// for a new frame. In that case, the renderer should wait until the frame timeout to
    /// ask the embedder to repaint.
    pub(crate) fn wait_to_paint(&self, repaint_reason: RepaintReason) -> bool {
        if self.observers.borrow().is_empty() || repaint_reason != RepaintReason::NewWebRenderFrame
        {
            return false;
        }

        self.waiting_for_frame.load(Ordering::Relaxed)
    }
}

/// A [`RefreshDriverObserver`] is an internal subscriber to frame start signals from the
/// [`RefreshDriver`]. Examples of these kind of observers would be one that triggers new
/// animation frames right after vsync signals or one that handles touch interactions once
/// per frame.
pub(crate) trait RefreshDriverObserver {
    /// Informs the observer that a new frame has started. The observer should return
    /// `true` to keep observing or `false` if wants to stop observing and should be
    /// removed by the [`BaseRefreshDriver`].
    fn frame_started(&self, compositor: &mut Painter) -> bool;
}

/// The [`AnimationRefreshDriverObserver`] is the default implementation of a
/// [`RefreshDriver`] on systems without vsync hardware integration. It has a very basic
/// way of triggering frames using a timer. It prevents new animation frames until the
/// timer has fired.
pub(crate) struct AnimationRefreshDriverObserver {
    /// The channel on which messages can be sent to the Constellation.
    pub(crate) constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// Whether or not we are currently animating via a timer.
    pub(crate) animating: Cell<bool>,
}

impl AnimationRefreshDriverObserver {
    pub(crate) fn new(constellation_sender: Sender<EmbedderToConstellationMessage>) -> Self {
        Self {
            constellation_sender,
            animating: Default::default(),
        }
    }

    pub(crate) fn notify_animation_state_changed(
        &self,
        webview_renderer: &WebViewRenderer,
    ) -> bool {
        if !webview_renderer.animating() {
            // If no other WebView is animating we will officially stop animating once the
            // next frame has been painted.
            return false;
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
            return false;
        }

        self.animating.set(true);
        true
    }
}

impl RefreshDriverObserver for AnimationRefreshDriverObserver {
    fn frame_started(&self, compositor: &mut Painter) -> bool {
        // If any WebViews are animating ask them to paint again for another animation tick.
        let animating_webviews = compositor.animating_webviews();

        // If nothing is animating any longer, update our state and exit early without requesting
        // any new frames.
        if animating_webviews.is_empty() {
            self.animating.set(false);
            return false;
        }

        // Request new animation frames from all animating WebViews.
        if let Err(error) =
            self.constellation_sender
                .send(EmbedderToConstellationMessage::TickAnimation(
                    animating_webviews,
                ))
        {
            warn!("Sending tick to constellation failed ({error:?}).");
            return false;
        }

        self.animating.set(true);
        true
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
struct TimerRefreshDriver {
    sender: Sender<TimerThreadMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl Default for TimerRefreshDriver {
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

impl TimerRefreshDriver {
    fn queue_timer(&self, duration: Duration, callback: BoxedTimerCallback) {
        let _ = self
            .sender
            .send(TimerThreadMessage::Request(TimerEventRequest {
                callback,
                duration,
            }));
    }
}

impl RefreshDriver for TimerRefreshDriver {
    fn observe_next_frame(&self, new_start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        const FRAME_DURATION: Duration = Duration::from_millis(1000 / 120);
        self.queue_timer(FRAME_DURATION, new_start_frame_callback);
    }
}

impl Drop for TimerRefreshDriver {
    fn drop(&mut self) {
        let _ = self.sender.send(TimerThreadMessage::Quit);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}
