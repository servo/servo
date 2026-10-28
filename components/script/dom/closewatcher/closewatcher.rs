/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::root::DomRoot;

use crate::dom::abortsignal::AbortAlgorithm;
use crate::dom::bindings::codegen::Bindings::CloseWatcherBinding::{
    CloseWatcherMethods, CloseWatcherOptions,
};
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::eventtarget::EventTarget;
use crate::dom::internalclosewatcher::{InternalCloseWatcher, InternalCloseWatcherOwner};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CloseWatcher {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/#internal-close-watcher>
    #[conditional_malloc_size_of]
    internal_close_watcher: DomRefCell<Option<Rc<InternalCloseWatcher>>>,
}

impl CloseWatcher {
    pub(crate) fn new_inherited() -> CloseWatcher {
        Self {
            eventtarget: EventTarget::new_inherited(),
            internal_close_watcher: Default::default(),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<CloseWatcher> {
        reflect_dom_object_with_proto(cx, Box::new(CloseWatcher::new_inherited()), window, proto)
    }
}

impl CloseWatcherMethods<DomTypeHolder> for CloseWatcher {
    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        options: &CloseWatcherOptions,
    ) -> Fallible<DomRoot<CloseWatcher>> {
        // Step 1. If this's relevant global object's associated Document is not fully active, then
        // throw an "InvalidStateError" DOMException.
        if !window.Document().is_fully_active() {
            return Err(Error::InvalidState(Some("Failed to construct 'CloseWatcher': CloseWatchers cannot be created in detached Windows.".into())));
        }
        // Step 2. Let closeWatcher be the result of establishing a close watcher given this's
        // relevant global object, with:
        // - cancelAction given canPreventClose being to return the result of firing an event named
        //   cancel at this, with the cancelable attribute initialized to canPreventClose.
        // - closeAction being to fire an event named close at this.
        // - getEnabledState being to return true.
        let close_watcher = CloseWatcher::new_with_proto(cx, window, proto);
        let internal_close_watcher = InternalCloseWatcher::establish(
            cx.no_gc(),
            window,
            InternalCloseWatcherOwner::CloseWatcher(close_watcher.as_traced()),
        );
        // Step 3. If options["signal"] exists, then:
        if let Some(signal) = options.signal.as_ref() {
            // Step 3.1. If options["signal"] is aborted, then destroy closeWatcher.
            if signal.aborted() {
                internal_close_watcher.destroy(cx.no_gc())
            }
            // Step 3.2. Add the following steps to options["signal"]:
            // Step 3.2.1. Destroy closeWatcher.
            signal.add(&AbortAlgorithm::CloseWatcher(
                internal_close_watcher.clone(),
            ));
        }
        // Step 4. Set this's internal close watcher to closeWatcher.
        close_watcher
            .internal_close_watcher
            .safe_borrow_mut(cx.no_gc())
            .replace(internal_close_watcher);
        Ok(close_watcher)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-requestclose>
    fn RequestClose(&self, cx: &mut JSContext) {
        // > The requestClose() method steps are to request to close this's internal close watcher
        // > with false.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.request_to_close(cx, false);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-close>
    fn Close(&self, cx: &mut JSContext) {
        // > The close() method steps are to close this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.close(cx);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-destroy>
    fn Destroy(&self, cx: &mut JSContext) {
        // > The destroy() method steps are to destroy this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.destroy(cx.no_gc());
        }
    }

    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-oncancel>
    event_handler!(cancel, GetOncancel, SetOncancel);
    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-onclose>
    event_handler!(close, GetOnclose, SetOnclose);
}
