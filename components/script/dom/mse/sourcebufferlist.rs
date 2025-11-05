use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use stylo_atoms::Atom;
use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::SourceBufferListBinding::SourceBufferListMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::sourcebuffer::SourceBuffer;

#[dom_struct]
pub(crate) struct SourceBufferList {
    event_target: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    inner: DomRefCell<Box<dyn servo_media::mse::SourceBufferList>>,
}

impl SourceBufferList {
    pub(crate) fn new_inherited(
        inner: DomRefCell<Box<dyn servo_media::mse::SourceBufferList>>,
    ) -> Self {
        let this = Self {
            event_target: EventTarget::new_inherited(),
            inner,
        };
        // {
        //     let (sen, rec) = ipc::channel().unwrap();
        //     this.inner.borrow().on_add_source_buffer(Box::new(move || {
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
        //                         Atom::from("on_add_source_buffer"),
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
    pub(crate) fn new(
        global: &GlobalScope,
        inner: DomRefCell<Box<dyn servo_media::mse::SourceBufferList>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(inner)), global, can_gc)
    }
}

impl SourceBufferListMethods<crate::DomTypeHolder> for SourceBufferList {
    fn Length(&self) -> u32 {
        self.inner.borrow().length()
    }

    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<SourceBuffer>> {
        self.inner.borrow().index(index).map(|buffer| {
            let source_buffer =
                SourceBuffer::new(&*self.global(), DomRefCell::new(buffer), CanGc::note());
            source_buffer
        })
    }

    event_handler!(
        on_add_source_buffer,
        GetOnaddsourcebuffer,
        SetOnaddsourcebuffer
    );
    event_handler!(
        on_remove_source_buffer,
        GetOnremovesourcebuffer,
        SetOnremovesourcebuffer
    );
}
