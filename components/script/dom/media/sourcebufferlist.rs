/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::SourceBufferListBinding::SourceBufferListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::media::sourcebuffer::SourceBuffer;
use crate::dom::window::Window;

/// <https://w3c.github.io/media-source/#sourcebufferlist>
#[dom_struct]
pub(crate) struct SourceBufferList {
    eventtarget: EventTarget,
    buffers: DomRefCell<Vec<Dom<SourceBuffer>>>,
}

impl SourceBufferList {
    fn new_inherited() -> SourceBufferList {
        SourceBufferList {
            eventtarget: EventTarget::new_inherited(),
            buffers: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, window: &Window) -> DomRoot<SourceBufferList> {
        reflect_dom_object_with_cx(Box::new(SourceBufferList::new_inherited()), window, cx)
    }

    pub(crate) fn len(&self) -> usize {
        self.buffers.borrow().len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buffers.borrow().is_empty()
    }

    pub(crate) fn contains(&self, buffer: &SourceBuffer) -> bool {
        self.buffers.borrow().iter().any(|item| &**item == buffer)
    }

    pub(crate) fn item(&self, index: usize) -> Option<DomRoot<SourceBuffer>> {
        self.buffers
            .borrow()
            .get(index)
            .map(|buffer| DomRoot::from_ref(&**buffer))
    }

    /// The source buffers of this list, rooted so that callers can iterate them
    /// without holding a borrow of the list.
    pub(crate) fn buffers(&self) -> Vec<DomRoot<SourceBuffer>> {
        self.buffers
            .borrow()
            .iter()
            .map(|buffer| DomRoot::from_ref(&**buffer))
            .collect()
    }

    /// Appends `buffer` and queues the `addsourcebuffer` event required by the algorithms
    /// that mutate this list.
    pub(crate) fn add(&self, buffer: &SourceBuffer) {
        self.buffers.borrow_mut().push(Dom::from_ref(buffer));
        self.queue_event(Atom::from("addsourcebuffer"));
    }

    /// Removes `buffer` and queues the `removesourcebuffer` event. Does nothing when the
    /// buffer is not in the list.
    pub(crate) fn remove(&self, buffer: &SourceBuffer) {
        let mut buffers = self.buffers.borrow_mut();
        let Some(index) = buffers.iter().position(|item| &**item == buffer) else {
            return;
        };
        buffers.remove(index);
        drop(buffers);

        self.queue_event(Atom::from("removesourcebuffer"));
    }

    /// Replaces the contents of the list, queueing the events implied by the difference
    /// with the previous contents.
    ///
    /// This backs the `activeSourceBuffers` bookkeeping, which the specification defines
    /// as a list kept in the same order as `sourceBuffers`.
    pub(crate) fn update(&self, buffers: &[DomRoot<SourceBuffer>]) {
        let previous = self.buffers();
        let removed = previous
            .iter()
            .any(|buffer| !buffers.iter().any(|item| item == buffer));
        let added = buffers
            .iter()
            .any(|buffer| !previous.iter().any(|item| item == buffer));

        if !removed && !added {
            return;
        }

        *self.buffers.borrow_mut() = buffers
            .iter()
            .map(|buffer| Dom::from_ref(&**buffer))
            .collect();

        if removed {
            self.queue_event(Atom::from("removesourcebuffer"));
        }
        if added {
            self.queue_event(Atom::from("addsourcebuffer"));
        }
    }

    /// Empties the list without queueing any event, leaving it to the caller to queue the
    /// single event the detaching algorithm asks for.
    pub(crate) fn clear(&self) {
        self.buffers.borrow_mut().clear();
    }

    pub(crate) fn queue_removesourcebuffer_event(&self) {
        self.queue_event(Atom::from("removesourcebuffer"));
    }

    fn queue_event(&self, name: Atom) {
        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(fire_source_buffer_list_event: move |cx| {
                let this = this.root();
                this.upcast::<EventTarget>().fire_event(cx, name);
            }));
    }
}

impl SourceBufferListMethods<crate::DomTypeHolder> for SourceBufferList {
    /// <https://w3c.github.io/media-source/#dom-sourcebufferlist-length>
    fn Length(&self) -> u32 {
        self.len() as u32
    }

    /// <https://w3c.github.io/media-source/#dfn-sourcebufferlist-getter>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<SourceBuffer>> {
        self.item(index as usize)
    }

    // <https://w3c.github.io/media-source/#dom-sourcebufferlist-onaddsourcebuffer>
    event_handler!(addsourcebuffer, GetOnaddsourcebuffer, SetOnaddsourcebuffer);

    // <https://w3c.github.io/media-source/#dom-sourcebufferlist-onremovesourcebuffer>
    event_handler!(
        removesourcebuffer,
        GetOnremovesourcebuffer,
        SetOnremovesourcebuffer
    );
}
