use std::str::FromStr;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use dom_struct::dom_struct;
use js::gc::HandleObject;
use mime::Mime;
use script_bindings::codegen::GenericBindings::MediaSourceBinding::{
    EndOfStreamError, MediaSourceMethods, ReadyState,
};
use script_bindings::domstring::DOMString;
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use crate::dom::bindings::inheritance::Castable;

use servo_media::ServoMedia;
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::sourcebuffer::SourceBuffer;
use crate::dom::sourcebufferlist::SourceBufferList;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaSource {
    event_target: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    inner: Box<dyn servo_media::mse::MediaSource>,
}

impl MediaSource {
    pub(crate) fn new_inherited() -> Self {
        let this = MediaSource {
            event_target: EventTarget::new_inherited(),
            inner: ServoMedia::get().create_mse_source().unwrap(),
        };
        // Register events
        // Due to the fact on_source_open takes a closure that is Send+Sync we have to use a channel.
        // {
        //     let (sen, rec) = ipc::channel().unwrap();
        //     this.inner.on_source_open(Box::new(move || {
        //         let _ = sen.send(());
        //     }));
        //     let global = this.global();
        //     let task_source = global
        //         .task_manager()
        //         .dom_manipulation_task_source()
        //         .to_sendable();
        //
        //     ROUTER.add_typed_route(
        //         rec,
        //         Box::new({
        //             let trusted_self = Trusted::new(&this);
        //             move |_| {
        //                 let trusted_self = trusted_self.clone();
        //                 task_source.queue(task!(on_source_open: move || {
        //                     let this = trusted_self.root();
        //                     let global = this.global();
        //                     let event = Event::new(
        //                         &global,
        //                         Atom::from("source_open"),
        //                         EventBubbles::DoesNotBubble,
        //                         EventCancelable::NotCancelable,
        //                         CanGc::note()
        //                     );
        //                     event.fire(this.upcast(), CanGc::note());
        //                 }));
        //             }
        //         })
        //     );
        // }
        // {
        //     let (sen, rec) = ipc::channel().unwrap();
        //     this.inner.on_source_ended(Box::new(move || {
        //         let _ = sen.send(());
        //     }));
        //     let global = this.global();
        //     let task_source = global
        //         .task_manager()
        //         .dom_manipulation_task_source()
        //         .to_sendable();
        //
        //     ROUTER.add_typed_route(
        //         rec,
        //         Box::new({
        //             let trusted_self = Trusted::new(&this);
        //             move |_| {
        //                 let trusted_self = trusted_self.clone();
        //                 task_source.queue(task!(on_source_ended: move || {
        //                     let this = trusted_self.root();
        //                     let global = this.global();
        //                     let event = Event::new(
        //                         &global,
        //                         Atom::from("source_ended"),
        //                         EventBubbles::DoesNotBubble,
        //                         EventCancelable::NotCancelable,
        //                         CanGc::note()
        //                     );
        //                     event.fire(this.upcast(), CanGc::note());
        //                 }));
        //             }
        //         })
        //     );
        // }
        // {
        //     let (sen, rec) = ipc::channel().unwrap();
        //     this.inner.on_source_close(Box::new(move || {
        //         let _ = sen.send(());
        //     }));
        //     let global = this.global();
        //     let task_source = global
        //         .task_manager()
        //         .dom_manipulation_task_source()
        //         .to_sendable();
        //     ROUTER.add_typed_route(
        //         rec,
        //         Box::new({
        //             let trusted_self = Trusted::new(&this);
        //             move |_| {
        //                 let trusted_self = trusted_self.clone();
        //                 task_source.queue(task!(on_source_close: move || {
        //                     let this = trusted_self.root();
        //                     let global = this.global();
        //                     let event = Event::new(
        //                         &global,
        //                         Atom::from("source_close"),
        //                         EventBubbles::DoesNotBubble,
        //                         EventCancelable::NotCancelable,
        //                         CanGc::note()
        //                     );
        //                     event.fire(this.upcast(), CanGc::note());
        //                 }));
        //             }
        //         })
        //     );
        // }
        this
    }

    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(Self::new_inherited()), global, proto, can_gc)
    }

    /// Used by URL.createObjectURL
    pub(crate) fn get_source_url_id(&self) -> Uuid {
        self.global().get_source_url_id(&self)
    }

    pub(crate) fn inner(&self) -> &dyn servo_media::mse::MediaSource {
        self.inner.as_ref()
    }
}

impl MediaSourceMethods<crate::DomTypeHolder> for MediaSource {
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<MediaSource> {
        MediaSource::new_with_proto(global.as_global_scope(), proto, can_gc)
    }

    fn SourceBuffers(&self) -> DomRoot<SourceBufferList> {
        SourceBufferList::new(
            &*self.global(),
            DomRefCell::new(self.inner.source_buffers()),
            CanGc::note(),
        )
    }

    fn ActiveSourceBuffers(&self) -> DomRoot<SourceBufferList> {
        SourceBufferList::new(
            &*self.global(),
            DomRefCell::new(self.inner.active_source_buffers()),
            CanGc::note(),
        )
    }

    fn ReadyState(&self) -> ReadyState {
        match self.inner.ready_state() {
            servo_media::mse::ReadyState::Closed => ReadyState::Closed,
            servo_media::mse::ReadyState::Open => ReadyState::Open,
            servo_media::mse::ReadyState::Ended => ReadyState::Ended,
        }
    }

    fn Duration(&self) -> f64 {
        self.inner.duration().unwrap_or(f64::NAN)
    }

    fn SetDuration(&self, value: f64) {
        self.inner.set_duration(value)
    }

    fn CanConstructInDedicatedWorker(_global: &Window) -> bool {
        // TODO: Fix
        false
    }

    fn AddSourceBuffer(&self, type_: DOMString) -> DomRoot<SourceBuffer> {
        SourceBuffer::new(
            &*self.global(),
            DomRefCell::new(self.inner.add_source_buffer(&type_.to_string()).unwrap()),
            CanGc::note(),
        )
    }

    fn RemoveSourceBuffer(&self, source_buffer: &SourceBuffer) {
        self.inner.remove_source_buffer(&**source_buffer.inner())
    }

    fn EndOfStream(&self, error: Option<EndOfStreamError>) {
        self.inner.end_of_stream(error.map(|e| match e {
            EndOfStreamError::Network => servo_media::mse::EosError::Network,
            EndOfStreamError::Decode => servo_media::mse::EosError::Decode,
        }))
    }

    fn SetLiveSeekableRange(&self, start: Finite<f64>, end: Finite<f64>) {
        self.inner.set_live_seekable_range(Some(*start), Some(*end))
    }

    fn ClearLiveSeekableRange(&self) {
        self.inner.clear_live_seekable_range()
    }

    /// <https://www.w3.org/TR/media-source-2/#istypesupported-method>
    fn IsTypeSupported(_global: &Window, type_: DOMString) -> bool {
        // Step 1. If type is an empty string, then return false.
        let type_str = type_.to_string();
        if type_str.is_empty() {
            return false;
        }
        // Step 2. If type does not contain a valid MIME type string, then return false.
        let Ok(_mime_type) = Mime::from_str(&type_str) else {
            return false;
        };
        // TODO: fix remaining steps
        // Step 3. If type contains a media type or media subtype that the MediaSource does not support, then return false.
        // Step 4. If type contains a codec that the MediaSource does not support, then return false.
        // Step 5. If the MediaSource does not support the specified combination of media type, media subtype, and codecs then return false.
        // Step 6 Return true.
        match &*type_str {
            "video/mp4" => true,
            "audio/mp4" => true,
            _ => false,
        }
    }

    event_handler!(source_open, GetOnsourceopen, SetOnsourceopen);
    event_handler!(source_ended, GetOnsourceended, SetOnsourceended);
    event_handler!(source_close, GetOnsourceclose, SetOnsourceclose);
}
