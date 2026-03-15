/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use fonts::FontContextWebFontMethods;
use js::rust::HandleObject;
use script_bindings::like::Setlike;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FontFaceSetBinding::FontFaceSetMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::fontface::FontFace;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://drafts.csswg.org/css-font-loading/#FontFaceSet-interface>
#[dom_struct]
pub(crate) struct FontFaceSet {
    target: EventTarget,

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-readypromise-slot>
    #[conditional_malloc_size_of]
    promise: Rc<Promise>,

    set_entries: DomRefCell<Vec<Dom<FontFace>>>,
}

impl FontFaceSet {
    fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> Self {
        FontFaceSet {
            target: EventTarget::new_inherited(),
            promise: Promise::new(global, can_gc),
            set_entries: Default::default(),
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

    /// Fulfill the font ready promise, returning true if it was not already fulfilled beforehand.
    pub(crate) fn fulfill_ready_promise_if_needed(&self, can_gc: CanGc) -> bool {
        if self.promise.is_fulfilled() {
            return false;
        }
        self.promise.resolve_native(self, can_gc);
        true
    }

    pub(crate) fn waiting_to_fullfill_promise(&self) -> bool {
        !self.promise.is_fulfilled()
    }

    fn contains_face(&self, target: &FontFace) -> bool {
        self.set_entries
            .borrow()
            .iter()
            .any(|face| &**face == target)
    }

    /// Removes a face from the set's set entries.
    fn delete_face(&self, target: &FontFace) -> bool {
        let mut set_entries = self.set_entries.borrow_mut();
        let Some(index) = set_entries.iter().position(|face| &**face == target) else {
            return false;
        };
        set_entries.remove(index);
        true
    }
}

impl FontFaceSetMethods<crate::DomTypeHolder> for FontFaceSet {
    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-ready>
    fn Ready(&self) -> Rc<Promise> {
        self.promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-add>
    fn Add(&self, font_face: &FontFace) -> DomRoot<FontFaceSet> {
        // Step 1. If font is already in the FontFaceSet’s set entries,
        // skip to the last step of this algorithm immediately.
        if self.contains_face(font_face) {
            return DomRoot::from_ref(self);
        }

        // TODO: Step 2. If font is CSS-connected, throw an InvalidModificationError
        // exception and exit this algorithm immediately.

        // Step 3. Add the font argument to the FontFaceSet’s set entries.
        self.set_entries.borrow_mut().push(Dom::from_ref(font_face));
        font_face.set_associated_font_face_set(self);

        // Step 4. If font’s status attribute is "loading":
        // Step 4.1 If the FontFaceSet’s [[LoadingFonts]] list is empty, switch the FontFaceSet to loading.
        // Step 4.2 Append font to the FontFaceSet’s [[LoadingFonts]] list.
        self.handle_font_face_status_changed(font_face);

        // Step 5. Return the FontFaceSet.
        DomRoot::from_ref(self)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-delete>
    fn Delete(&self, to_delete: &FontFace) -> bool {
        // TODO Step 1. If font is CSS-connected, return false and exit this algorithm immediately.

        // Step 2. Let deleted be the result of removing font from the FontFaceSet’s set entries.
        // TODO: Step 3. If font is present in the FontFaceSet’s [[LoadedFonts]], or [[FailedFonts]] lists, remove it.
        // TODO: Step 4. If font is present in the FontFaceSet’s [[LoadingFonts]] list, remove it. If font was the last
        // item in that list (and so the list is now empty), switch the FontFaceSet to loaded.
        // Step 5. Return deleted.
        self.delete_face(to_delete)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-clear>
    fn Clear(&self) {
        // Step 1. Remove all non-CSS-connected items from the FontFaceSet’s set entries,
        // its [[LoadedFonts]] list, and its [[FailedFonts]] list.
        self.set_entries.borrow_mut().clear();

        // TODO Step 2. If the FontFaceSet’s [[LoadingFonts]] list is non-empty, remove all items from it,
        // then switch the FontFaceSet to loaded.
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

    /// <https://html.spec.whatwg.org/multipage/#customstateset>
    fn Size(&self) -> u32 {
        self.set_entries.borrow().len() as u32
    }
}

impl Setlike for FontFaceSet {
    type Key = DomRoot<FontFace>;

    #[inline(always)]
    fn get_index(&self, index: u32) -> Option<Self::Key> {
        self.set_entries
            .borrow()
            .get(index as usize)
            .map(|face| face.as_rooted())
    }

    #[inline(always)]
    fn size(&self) -> u32 {
        self.set_entries.borrow().len() as u32
    }

    #[inline(always)]
    fn add(&self, face: Self::Key) {
        self.set_entries.borrow_mut().push(face.as_traced());
    }

    #[inline(always)]
    fn has(&self, target: Self::Key) -> bool {
        self.contains_face(&target)
    }

    #[inline(always)]
    fn clear(&self) {
        self.set_entries.borrow_mut().clear();
    }

    #[inline(always)]
    fn delete(&self, to_delete: Self::Key) -> bool {
        self.delete_face(&to_delete)
    }
}
