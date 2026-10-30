/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use embedder_traits::EventLoopWaker;
use rustc_hash::FxHashMap;
use servo_base::id::WebViewId;
use servo_config::prefs;
use webrender_api::{ColorF, PropertyBindingKey, PropertyValue};

use crate::refresh_driver::TimerRefreshDriver;
use crate::webview_renderer::WebViewRenderer;

/// The amount of the time the caret blinks before ceasing, in order to preserve power. User
/// activity (a new display list) will reset this.
///
/// TODO: This should be controlled by system settings.
pub(crate) const CARET_BLINK_TIMEOUT: Duration = Duration::from_secs(30);

/// A struct responsible for managing paint-side animations. Currently this only handles text caret
/// blinking, but the idea is that in the future this would handle other types of paint-side
/// animations as well.
///
/// Note: This does not control animations requiring layout (all CSS transitions and animations
/// currently) nor animations due to touch events such as fling.
pub(crate) struct WebContentAnimator {
    event_loop_waker: Box<dyn EventLoopWaker>,
    timer_refresh_driver: Rc<TimerRefreshDriver>,
    caret_visible: Cell<bool>,
    timer_scheduled: Cell<bool>,
    need_update: Arc<AtomicBool>,
}

impl WebContentAnimator {
    pub(crate) fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        timer_refresh_driver: Rc<TimerRefreshDriver>,
    ) -> Self {
        Self {
            event_loop_waker,
            timer_refresh_driver,
            caret_visible: Cell::new(true),
            timer_scheduled: Default::default(),
            need_update: Default::default(),
        }
    }

    pub(crate) fn schedule_timer_if_necessary(&self) {
        if self.timer_scheduled.get() {
            return;
        }

        let Some(caret_blink_time) = prefs::get().editing_caret_blink_time() else {
            return;
        };

        let event_loop_waker = self.event_loop_waker.clone();
        let need_update = self.need_update.clone();
        self.timer_refresh_driver.queue_timer(
            caret_blink_time,
            Box::new(move || {
                need_update.store(true, Ordering::Relaxed);
                event_loop_waker.wake();
            }),
        );
        self.timer_scheduled.set(true);
    }

    /// Trigger a potential update to this [`WebContentAnimator`], returning `true` if
    /// anything was updated and `false` otherwise.
    pub(crate) fn update(&self, webview_renderers: &FxHashMap<WebViewId, WebViewRenderer>) -> bool {
        if !self.need_update.load(Ordering::Relaxed) {
            return false;
        }

        let mut anything_updated = false;
        for renderer in webview_renderers.values() {
            renderer.for_each_connected_pipeline(&mut |pipeline_details| {
                anything_updated = pipeline_details.animations.update() || anything_updated;
            });
        }

        self.timer_scheduled.set(false);
        self.need_update.store(false, Ordering::Relaxed);

        if !anything_updated {
            // All animations have stopped. When a new blinking caret is activated we want
            // it to start in the visible state, so we set `caret_visible` to true here.
            self.caret_visible.set(true);
            return false;
        }

        self.caret_visible.set(!self.caret_visible.get());
        self.schedule_timer_if_necessary();
        true
    }

    pub(crate) fn values(
        &self,
        webview_renderers: &FxHashMap<WebViewId, WebViewRenderer>,
    ) -> Vec<PropertyValue<ColorF>> {
        let mut colors = Vec::new();
        for renderer in webview_renderers.values() {
            renderer.for_each_connected_pipeline(&mut |pipeline_details| {
                if let Some(property_value) =
                    pipeline_details.animations.values(self.caret_visible.get())
                {
                    colors.push(property_value);
                }
            });
        }
        colors
    }
}

/// This structure tracks the animations active for a given pipeline. Currently only caret
/// blinking is tracked, but in the future this could perhaps track paint-side animations.
#[derive(Default)]
pub(crate) struct PipelineAnimations {
    caret: RefCell<Option<CaretAnimation>>,
}

impl PipelineAnimations {
    /// Update this [`PipelineAnimations`], returning `true` if anything updated and
    /// `false` otherwise.
    pub(crate) fn update(&self) -> bool {
        let mut maybe_caret = self.caret.borrow_mut();
        let Some(caret) = maybe_caret.as_mut() else {
            return false;
        };
        caret.update();
        if caret.done() {
            *maybe_caret = None;
        }
        true
    }

    pub(crate) fn values(&self, caret_visible: bool) -> Option<PropertyValue<ColorF>> {
        let maybe_caret = self.caret.borrow();
        Some(maybe_caret.as_ref()?.value(caret_visible))
    }

    pub(crate) fn handle_new_display_list(
        &self,
        caret_property_binding: Option<(PropertyBindingKey<ColorF>, ColorF)>,
        web_content_animator: &WebContentAnimator,
    ) {
        let Some(caret_blink_time) = prefs::get().editing_caret_blink_time() else {
            return;
        };

        *self.caret.borrow_mut() = match caret_property_binding {
            Some((caret_property_key, original_caret_color)) => {
                web_content_animator.schedule_timer_if_necessary();
                Some(CaretAnimation {
                    caret_property_key,
                    original_caret_color,
                    remaining_blink_count: (CARET_BLINK_TIMEOUT.as_millis() /
                        caret_blink_time.as_millis())
                        as usize,
                })
            },
            None => None,
        }
    }
}

/// Tracks the state of an ongoing caret blinking animation.
struct CaretAnimation {
    pub caret_property_key: PropertyBindingKey<ColorF>,
    pub original_caret_color: ColorF,
    pub remaining_blink_count: usize,
}

impl CaretAnimation {
    pub(crate) fn done(&self) -> bool {
        self.remaining_blink_count == 0
    }

    pub(crate) fn update(&mut self) {
        self.remaining_blink_count = self.remaining_blink_count.saturating_sub(1);
    }

    pub(crate) fn value(&self, caret_visible: bool) -> PropertyValue<ColorF> {
        debug_assert!(self.remaining_blink_count != 0);
        let value = if caret_visible {
            self.original_caret_color
        } else {
            ColorF::TRANSPARENT
        };
        PropertyValue {
            key: self.caret_property_key,
            value,
        }
    }
}
