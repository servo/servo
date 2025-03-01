/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use fonts::FontContextWebFontMethods;
use js::rust::HandleObject;

use super::bindings::reflector::DomGlobal;
use super::types::Window;
use crate::dom::bindings::codegen::Bindings::FontFaceSetBinding::FontFaceSetMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::fontface::FontFace;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

/// <https://drafts.csswg.org/css-font-loading/#FontFaceSet-interface>
#[dom_struct]
pub(crate) struct FontFaceSet {
    target: EventTarget,

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-readypromise-slot>
    #[ignore_malloc_size_of = "Rc"]
    promise: Rc<Promise>,
}

impl FontFaceSet {
    fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> Self {
        FontFaceSet {
            target: EventTarget::new_inherited(),
            promise: Promise::new(global, can_gc),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(FontFaceSet::new_inherited(global, can_gc)),
            global,
            proto,
            can_gc,
        )
    }

    pub(super) fn handle_font_face_status_changed(&self, font_face: &FontFace) {
        if font_face.loaded() {
            let Some(window) = DomRoot::downcast::<Window>(self.global()) else {
                return;
            };

            let (family_name, template) = font_face
                .template()
                .expect("A loaded web font should have a template");
            window
                .font_context()
                .add_template_to_font_context(family_name, template);
            window.Document().dirty_all_nodes();
        }
    }

    pub(crate) fn fulfill_ready_promise_if_needed(&self, can_gc: CanGc) {
        if !self.promise.is_fulfilled() {
            self.promise.resolve_native(self, can_gc);
        }
    }
}

impl FontFaceSetMethods<crate::DomTypeHolder> for FontFaceSet {
    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-ready>
    fn Ready(&self) -> Rc<Promise> {
        self.promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-add>
    fn Add(&self, font_face: &FontFace) -> DomRoot<FontFaceSet> {
        font_face.set_associated_font_face_set(self);
        self.handle_font_face_status_changed(font_face);
        DomRoot::from_ref(self)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-load>
    fn Load(&self, _font: DOMString, _text: DOMString, can_gc: CanGc) -> Rc<Promise> {
        // Step 1. Let font face set be the FontFaceSet object this method was called on. Let
        // promise be a newly-created promise object.
        let promise = Promise::new(&self.global(), can_gc);

        // TODO: Step 3. Find the matching font faces from font face set using the font and text
        // arguments passed to the function, and let font face list be the return value (ignoring
        // the found faces flag). If a syntax error was returned, reject promise with a SyntaxError
        // exception and terminate these steps.

        let trusted = TrustedPromise::new(promise.clone());
        // Step 4. Queue a task to run the following steps synchronously:
        self.global()
            .task_manager()
            .font_loading_task_source()
            .queue(task!(resolve_font_face_set_load_task: move || {
                let promise = trusted.root();

                // TODO: Step 4.1. For all of the font faces in the font face list, call their load()
                // method.

                // TODO: Step 4.2. Resolve promise with the result of waiting for all of the
                // [[FontStatusPromise]]s of each font face in the font face list, in order.
                let matched_fonts = Vec::<&FontFace>::new();
                promise.resolve_native(&matched_fonts, CanGc::note());
            }));

        // Step 2. Return promise. Complete the rest of these steps asynchronously.
        promise
    }
}
