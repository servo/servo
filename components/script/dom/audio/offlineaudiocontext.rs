/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::Builder;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_base::id::PipelineId;
use servo_media::audio::context::OfflineAudioContextOptions as ServoMediaOfflineAudioContextOptions;

use crate::dom::audio::audiobuffer::{AudioBuffer, MAX_SAMPLE_RATE, MIN_SAMPLE_RATE};
use crate::dom::audio::audionode::MAX_CHANNEL_COUNT;
use crate::dom::audio::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use crate::dom::audio::offlineaudiocompletionevent::OfflineAudioCompletionEvent;
use crate::dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContext_Binding::BaseAudioContextMethods;
use crate::dom::bindings::codegen::Bindings::OfflineAudioContextBinding::{
    OfflineAudioContextMethods, OfflineAudioContextOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::promise::Promise;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct OfflineAudioContext {
    context: BaseAudioContext,
    channel_count: u32,
    length: u32,
    rendering_started: Cell<bool>,
    #[conditional_malloc_size_of]
    pending_rendering_promise: DomRefCell<Option<Rc<Promise>>>,
}

impl OfflineAudioContext {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_inherited(
        channel_count: u32,
        length: u32,
        sample_rate: f32,
        pipeline_id: PipelineId,
    ) -> Fallible<OfflineAudioContext> {
        let options = ServoMediaOfflineAudioContextOptions {
            channels: channel_count as u8,
            length: length as usize,
            sample_rate,
        };
        let context = BaseAudioContext::new_inherited(
            BaseAudioContextOptions::OfflineAudioContext(options),
            pipeline_id,
        )?;
        Ok(OfflineAudioContext {
            context,
            channel_count,
            length,
            rendering_started: Cell::new(false),
            pending_rendering_promise: Default::default(),
        })
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        channel_count: u32,
        length: u32,
        sample_rate: f32,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        if channel_count > MAX_CHANNEL_COUNT ||
            channel_count == 0 ||
            length == 0 ||
            !(MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate)
        {
            return Err(Error::NotSupported(None));
        }
        let pipeline_id = window.pipeline_id();
        let context =
            OfflineAudioContext::new_inherited(channel_count, length, sample_rate, pipeline_id)?;
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(context),
            window,
            proto,
        ))
    }
}

impl OfflineAudioContextMethods<crate::DomTypeHolder> for OfflineAudioContext {
    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-offlineaudiocontext>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        options: &OfflineAudioContextOptions,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        OfflineAudioContext::new(
            cx,
            window,
            proto,
            options.numberOfChannels,
            options.length,
            *options.sampleRate,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-offlineaudiocontext-numberofchannels-length-samplerate>
    fn Constructor_(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        number_of_channels: u32,
        length: u32,
        sample_rate: Finite<f32>,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        OfflineAudioContext::new(cx, window, proto, number_of_channels, length, *sample_rate)
    }

    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-oncomplete
    event_handler!(complete, GetOncomplete, SetOncomplete);

    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-length>
    fn Length(&self) -> u32 {
        self.length
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-startrendering>
    fn StartRendering(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);
        if self.rendering_started.get() {
            promise.reject_error(cx, Error::InvalidState(None));
            return promise;
        }
        self.rendering_started.set(true);

        *self.pending_rendering_promise.safe_borrow_mut(cx.no_gc()) = Some(promise.clone());

        let processed_audio = Arc::new(Mutex::new(Vec::new()));
        let processed_audio_ = processed_audio.clone();
        let (sender, receiver) = mpsc::channel();
        let sender = Mutex::new(sender);
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .set_eos_callback(Box::new(move |buffer| {
                processed_audio_
                    .lock()
                    .unwrap()
                    .extend_from_slice((*buffer).as_ref());
                let _ = sender.lock().unwrap().send(());
            }));

        let this = Trusted::new(self);
        let task_source = self
            .global()
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        Builder::new()
            .name("OfflineACResolver".to_owned())
            .spawn(move || {
                let _ = receiver.recv();
                task_source.queue(task!(resolve: move |cx| {
                    let this = this.root();
                    let processed_audio = processed_audio.lock().unwrap();
                    let mut processed_audio: Vec<_> = processed_audio
                        .chunks(this.length as usize)
                        .map(|channel| channel.to_vec())
                        .collect();
                    // it can end up being empty if the task failed
                    if processed_audio.len() != this.length as usize {
                        processed_audio.resize(this.length as usize, Vec::new())
                    }
                    let buffer = AudioBuffer::new(
                        cx,
                        this.global().as_window(),
                        this.channel_count,
                        this.length,
                        *this.context.SampleRate(),
                        Some(processed_audio.as_slice()),
                    );
                    let promise = {
                        (*this
                            .pending_rendering_promise
                            .safe_borrow_mut(cx.no_gc()))
                            .take()
                            .unwrap()
                    };
                    promise.resolve_native(cx, &buffer);
                    let global = &this.global();
                    let window = global.as_window();
                    let event = OfflineAudioCompletionEvent::new(cx, window,
                                                                 atom!("complete"),
                                                                 EventBubbles::DoesNotBubble,
                                                                 EventCancelable::NotCancelable,
                                                                 &buffer);
                    event.upcast::<Event>().fire(cx, this.upcast());
                }));
            })
            .unwrap();

        if self
            .context
            .audio_context_impl()
            .lock()
            .unwrap()
            .resume()
            .is_none()
        {
            promise.reject_error(
                cx,
                Error::Type(c"Could not start offline rendering".to_owned()),
            );
        }

        promise
    }
}
