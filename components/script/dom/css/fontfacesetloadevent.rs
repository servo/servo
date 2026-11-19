/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::{HandleObject, MutableHandleValue};
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::FontFaceSetBinding::{
    FontFaceSetLoadEventInit, FontFaceSetLoadEventMethods,
};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::fontface::FontFace;
use crate::dom::globalscope::GlobalScope;

/// <https://drafts.csswg.org/css-font-loading/#fontfacesetloadevent>
#[dom_struct]
pub(crate) struct FontFaceSetLoadEvent {
    event: Event,

    /// The fonts that this event is about.
    fontfaces: Vec<Dom<FontFace>>,

    /// The JavaScript version of `fontfaces`, made once and then reused.
    #[ignore_malloc_size_of = "mozjs"]
    frozen_fontfaces: CachedFrozenArray,
}

impl FontFaceSetLoadEvent {
    fn new_inherited(fontfaces: &[DomRoot<FontFace>]) -> FontFaceSetLoadEvent {
        FontFaceSetLoadEvent {
            event: Event::new_inherited(),
            fontfaces: fontfaces
                .iter()
                .map(|fontface| Dom::from_ref(&**fontface))
                .collect(),
            frozen_fontfaces: CachedFrozenArray::new(),
        }
    }

    /// Creates a `FontFaceSetLoadEvent` from Servo's own code (not from JavaScript).
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        type_: Atom,
        fontfaces: &[DomRoot<FontFace>],
    ) -> DomRoot<FontFaceSetLoadEvent> {
        // Font load events don't bubble and can't be cancelled.
        Self::new_with_proto(cx, global, None, type_, false, false, fontfaces)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        fontfaces: &[DomRoot<FontFace>],
    ) -> DomRoot<FontFaceSetLoadEvent> {
        let event = reflect_dom_object_with_proto(
            cx,
            Box::new(FontFaceSetLoadEvent::new_inherited(fontfaces)),
            global,
            proto,
        );
        event
            .upcast::<Event>()
            .init_event(type_, bubbles, cancelable);
        event
    }
}

impl FontFaceSetLoadEventMethods<crate::DomTypeHolder> for FontFaceSetLoadEvent {
    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfacesetloadevent-fontfacesetloadevent>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &FontFaceSetLoadEventInit,
    ) -> DomRoot<FontFaceSetLoadEvent> {
        let fontfaces: Vec<DomRoot<FontFace>> = init
            .fontfaces
            .iter()
            .map(|fontface| DomRoot::from_ref(&**fontface))
            .collect();
        FontFaceSetLoadEvent::new_with_proto(
            cx,
            global,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &fontfaces,
        )
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfacesetloadevent-fontfaces>
    fn Fontfaces(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        self.frozen_fontfaces.get_or_init(
            cx,
            || {
                self.fontfaces
                    .iter()
                    .map(|fontface| DomRoot::from_ref(&**fontface))
                    .collect()
            },
            retval,
        );
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
