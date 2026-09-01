/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use cssparser::{Parser, ParserInput, UnicodeRange};
use dom_struct::dom_struct;
use fonts::FontFaceRuleInfo;
use js::context::JSContext;
use js::gc::Handle;
use js::jsapi::Value;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use layout_api::{QueryMsg, ReflowGoal};
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::FontFaceBinding::{
    FontFaceLoadStatus, FontFaceMethods,
};
use script_bindings::like::Setlike;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_arc::Arc as ServoArc;
use style::font_face::FamilyName;
use style::properties::shorthands::font;
use style::stylesheets::CssRuleType;
use style::values::computed::font::{FontFamilyList, SingleFontFamily};
use style::values::specified::font as specified_font;
use style_traits::ParsingMode;

use crate::css::css::parser_context_for_document;
use crate::dom::bindings::codegen::Bindings::FontFaceSetBinding::FontFaceSetMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::eventtarget::EventTarget;
use crate::dom::fontface::FontFace;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::Callback;
use crate::dom::types::PromiseNativeHandler;
use crate::dom::window::Window;
use crate::realms::enter_auto_realm;

/// <https://drafts.csswg.org/css-font-loading/#FontFaceSet-interface>
#[dom_struct]
pub(crate) struct FontFaceSet {
    target: EventTarget,

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-readypromise-slot>
    #[conditional_malloc_size_of]
    promise: RefCell<Rc<Promise>>,

    set_entries: DomRefCell<Vec<Dom<FontFace>>>,
}

impl FontFaceSet {
    fn new_inherited(promise: Rc<Promise>) -> Self {
        FontFaceSet {
            target: EventTarget::new_inherited(),
            promise: promise.into(),
            set_entries: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        let promise = Promise::new(cx, global);
        reflect_dom_object_with_proto(
            cx,
            Box::new(FontFaceSet::new_inherited(promise)),
            global,
            proto,
        )
    }

    pub(super) fn handle_font_face_status_changed(&self, cx: &mut JSContext, font_face: &FontFace) {
        match font_face.Status() {
            FontFaceLoadStatus::Loading => {
                self.switch_to_loading(cx);
            },
            FontFaceLoadStatus::Loaded => {
                let Some(window) = DomRoot::downcast::<Window>(self.global()) else {
                    return;
                };

                let (family_name, template) = font_face
                    .template()
                    .expect("A loaded web font should have a template");
                window
                    .font_context()
                    .add_template_to_font_context(family_name, template);
                window.Document().dirty_all_nodes(cx.no_gc());
            },
            _ => {},
        }
    }

    /// Fulfill the font ready promise, returning true if it was not already fulfilled beforehand.
    pub(crate) fn fulfill_ready_promise_if_needed(&self, cx: &mut JSContext) -> bool {
        let promise = self.promise.borrow().clone();
        if promise.is_fulfilled() {
            return false;
        }
        promise.resolve_native(cx, self);
        true
    }

    pub(crate) fn waiting_to_fullfill_promise(&self) -> bool {
        !self.promise.borrow().is_fulfilled()
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

    /// <https://drafts.csswg.org/css-font-loading/#switch-the-fontfaceset-to-loading>
    pub(crate) fn switch_to_loading(&self, cx: &mut JSContext) {
        // Step 1. Let font face set be the given FontFaceSet.
        // Note: This is self.

        // Step 2. Set the status attribute of font face set to "loading".
        // TODO: Implement the FontFaceSet status attribute.

        // Step 3. If font face set’s [[ReadyPromise]] slot currently holds a fulfilled
        // promise, replace it with a fresh pending promise.
        if self.promise.borrow().is_fulfilled() {
            *self.promise.borrow_mut() = Promise::new(cx, &self.global());
        }

        // Step 4. Queue a task to fire a font load event named loading at font face set.
        // TODO: Implement support for font loading events.
    }

    /// Runs the CSS cascade to ensure that new `@font-face` rules have
    /// an entry in this set.
    fn flush_author_font_set(&self, cx: &mut JSContext) {
        // FIXME: Use a new sort of ReflowGoal that only runs the CSS cascade without
        //        building a new box tree or running any sort of layout really.
        //        We query for the box area here, but we're not interested in the result.
        // FIXME: Figure out what to do for worker scopes.
        if let Some(window) = DomRoot::downcast::<Window>(self.global()) {
            let document = window.Document();
            if document.stylesheets_changed_since_last_reflow() {
                window.reflow(cx, ReflowGoal::LayoutQuery(QueryMsg::BoxArea));
            }
        }
    }

    /// Marks the entries corresponding to removed `@font-face` rules as not [css-connected].
    ///
    /// [css-connected]: https://drafts.csswg.org/css-font-loading/#css-connected
    pub(crate) fn notify_font_face_rules_removed(
        &self,
        removed_font_face_rules: &[ServoArc<FontFaceRuleInfo>],
    ) {
        let entries = self.set_entries.borrow_mut();
        for removed_font_face_rule in removed_font_face_rules {
            let Some(matching_font_face_object) = entries
                .iter()
                .find(|entry| entry.is_connected_to_font_face_rule(removed_font_face_rule))
            else {
                if cfg!(debug_assertions) {
                    unreachable!("Removed @font-face that was not previously present");
                }
                log::warn!("Removed @font-face that was not previously present");
                continue;
            };

            // https://drafts.csswg.org/css-font-loading/#font-face-css-connection:
            // > If a @font-face rule is removed from the document, its corresponding FontFace object is no longer CSS-connected.
            // > The connection is not restorable by any means.
            matching_font_face_object.disconnect_from_css();
        }
    }

    /// Uses the font matching rules to select font faces within `self` that can be used to
    /// render the provided text.
    ///
    /// This is used to implement Step 6 of
    /// <https://drafts.csswg.org/css-font-loading/#find-the-matching-font-faces>.
    fn query_fonts(&self, target_family: &FamilyName, sample_text: &str) -> Vec<DomRoot<FontFace>> {
        let mut matching_fonts = Vec::default();
        for font in self.set_entries.borrow().iter() {
            let font_face_rule = font.css_font_face_rule();
            let Some(font_face_rule) = font_face_rule.as_ref() else {
                // FIXME: Don't ignore font faces that are not css-connected here.
                continue;
            };

            if font_face_rule
                .descriptors
                .font_family
                .as_ref()
                .is_none_or(|family| family != target_family)
            {
                continue;
            }

            if font_face_rule
                .descriptors
                .unicode_range
                .as_ref()
                .is_some_and(|ranges| !any_character_in_any_unicode_range(sample_text, ranges))
            {
                continue;
            }

            // FIXME: Check other fields (weight, style, ...) here too. We need to investigate what other
            // browsers are doing, because at this point the font isn't actually loaded yet,
            // so the full descriptor is not available.
            matching_fonts.push(font.as_rooted());
        }

        matching_fonts
    }

    /// <https://drafts.csswg.org/css-font-loading/#find-the-matching-font-faces>
    fn find_the_matching_font_faces(
        &self,
        document: &Document,
        font: &str,
        sample_text: &str,
    ) -> Result<Vec<DomRoot<FontFace>>, FontQuerySyntaxError> {
        // Step 1. (Parse "font") and Step 2. (Unpack font shorthand) are implemented
        // in FontQueryParameters::parse.
        let parameters = FontQueryParameters::parse(document, font)?;

        // Step 2. If text was not explicitly provided, let it be a string containing a
        // single space character (U+0020 SPACE).
        // Note: "text" is not optional in our implementation yet.

        // Step 4. Let available font faces be the available font faces within source.
        // If the allow system fonts flag is specified, add all system fonts to available font faces.

        // Step 5. Let matched font faces initially be an empty list.
        let mut matched_faces = vec![];

        // Step 6. For each family in font family list, use the font matching rules to select the font faces
        // from available font faces that match the font style, and add them to matched font faces.
        // The use of the unicodeRange attribute means that this may be more than just a single font face.
        // Step 7. If matched font faces is empty, set the found faces flag to false. Otherwise, set it to true.
        // Note We don't need this yet.
        // Step 8. For each font face in matched font faces, if its defined unicode-range does not include the
        // codepoint of at least one character in text, remove it from the list.
        for family in parameters.families.list.iter() {
            let SingleFontFamily::FamilyName(target_family) = family else {
                continue; // Skip generic font faces
            };

            matched_faces.extend_from_slice(&self.query_fonts(target_family, sample_text));
        }

        // Step 9. Return matched font faces and the found faces flag.
        Ok(matched_faces)
    }
}

impl FontFaceSetMethods<crate::DomTypeHolder> for FontFaceSet {
    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-ready>
    fn Ready(&self, cx: &mut JSContext) -> Rc<Promise> {
        if self.promise.borrow().is_fulfilled() {
            // There may be pending style changes that cause new web fonts to start loading,
            // re-initializing document.fonts.ready.
            self.flush_author_font_set(cx);
        }
        self.promise.borrow().clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-add>
    fn Add(&self, cx: &mut JSContext, font_face: &FontFace) -> Fallible<DomRoot<FontFaceSet>> {
        // Step 1. If font is already in the FontFaceSet’s set entries,
        // skip to the last step of this algorithm immediately.
        if self.contains_face(font_face) {
            return Ok(DomRoot::from_ref(self));
        }

        // Step 2. If font is CSS-connected, throw an InvalidModificationError
        // exception and exit this algorithm immediately.
        if font_face.is_css_connected() {
            return Err(Error::InvalidModification(Some(
                "Cannot add CSS-connected FontFace to FontFaceSet".to_owned(),
            )));
        }

        // Step 3. Add the font argument to the FontFaceSet’s set entries.
        self.set_entries.borrow_mut().push(Dom::from_ref(font_face));
        font_face.set_associated_font_face_set(self);

        // Step 4. If font’s status attribute is "loading":
        // Step 4.1 If the FontFaceSet’s [[LoadingFonts]] list is empty, switch the FontFaceSet to loading.
        // Step 4.2 Append font to the FontFaceSet’s [[LoadingFonts]] list.
        self.handle_font_face_status_changed(cx, font_face);

        // Step 5. Return the FontFaceSet.
        Ok(DomRoot::from_ref(self))
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-delete>
    fn Delete(&self, to_delete: &FontFace) -> bool {
        // Step 1. If font is CSS-connected, return false and exit this algorithm immediately.
        if to_delete.is_css_connected() {
            return false;
        }

        // Step 2. Let deleted be the result of removing font from the FontFaceSet’s set entries.
        // TODO: Step 3. If font is present in the FontFaceSet’s [[LoadedFonts]], or [[FailedFonts]] lists, remove it.
        // TODO: Step 4. If font is present in the FontFaceSet’s [[LoadingFonts]] list, remove it. If font was the last
        // item in that list (and so the list is now empty), switch the FontFaceSet to loaded.
        // Step 5. Return deleted.
        self.delete_face(to_delete)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-clear>
    fn Clear(&self, cx: &mut JSContext) {
        self.flush_author_font_set(cx);

        // Step 1. Remove all non-CSS-connected items from the FontFaceSet’s set entries,
        // its [[LoadedFonts]] list, and its [[FailedFonts]] list.
        self.set_entries.borrow_mut().clear();

        // TODO Step 2. If the FontFaceSet’s [[LoadingFonts]] list is non-empty, remove all items from it,
        // then switch the FontFaceSet to loaded.
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-load>
    fn Load(&self, cx: &mut JSContext, font: DOMString, text: DOMString) -> Rc<Promise> {
        // Step 1. Let font face set be the FontFaceSet object this method was called on. Let
        // promise be a newly-created promise object.
        let load_promise = Promise::new(cx, &self.global());

        // Step 2. Return promise. Complete the rest of these steps asynchronously.
        #[derive(MallocSizeOf, JSTraceable)]
        #[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
        struct LoadPromiseFulfillmentHandler {
            /// The font faces that this should wait on.
            ///
            /// (Our current implementation waits for `document.fonts.ready` instead)
            font_face_objects: Vec<Dom<FontFace>>,

            #[conditional_malloc_size_of]
            load_promise: Rc<Promise>,
        }
        impl Callback for LoadPromiseFulfillmentHandler {
            fn callback(&self, cx: &mut CurrentRealm, _: Handle<Value>) {
                let font_face_objects: Vec<DomRoot<FontFace>> = self
                    .font_face_objects
                    .iter()
                    .map(|font_face| font_face.as_rooted())
                    .collect();
                self.load_promise.resolve_native(cx, &font_face_objects);
            }
        }

        // Step 4. Queue a task to run the following steps synchronously:
        let trusted_this = Trusted::new(self);
        let trusted_load_promise = TrustedPromise::new(load_promise.clone());
        let font = font.to_string();
        let text = text.to_string();
        self.global()
            .task_manager()
            .font_loading_task_source()
            .queue(task!(resolve_font_face_set_load_task: move |cx| {
                let load_promise = trusted_load_promise.root();
                let this = trusted_this.root();

                // This will need adjustments once FontFaceSet is exposed to workers.
                let Some(window) = DomRoot::downcast::<Window>(this.global()) else {
                    log::error!("FontFaceSet should not be exposed to non-window globals");
                    return;
                };
                let document = window.Document();

                // Step 3. Find the matching font faces from font face set using the font and text
                // arguments passed to the function, and let font face list be the return value (ignoring
                // the found faces flag). If a syntax error was returned, reject promise with a SyntaxError
                // exception and terminate these steps.
                let Ok(font_face_objects) = this.find_the_matching_font_faces(&document, &font, &text) else {
                    load_promise.reject_error(cx, Error::Syntax(Some("Failed to parse font query".into())));
                    return;
                };

                // Step 4.1. For all of the font faces in the font face list, call their load()
                // method.
                // Step 4.2. Resolve promise with the result of waiting for all of the
                // [[FontStatusPromise]]s of each font face in the font face list, in order.
                //
                // TODO: These steps are not implemented. Instead we wait until all fonts
                // are loaded by resolving the returned promise when
                // `document.fonts.ready` is resolved. The return list of fonts will not
                // be correct, but any code that waits on the promise will have
                // conservatively consistent behavior. This is important for preventing
                // intermittent results in WPT tests.
                let global = this.global();
                let handler = PromiseNativeHandler::new(
                    cx,
                    &global,
                    Some(Box::new(LoadPromiseFulfillmentHandler {
                        font_face_objects: font_face_objects.into_iter().map(|font_face| font_face.as_traced()).collect(),
                        load_promise,
                    })),
                    None,
                );

                let ready_promise = this.Ready(cx);
                let mut realm = enter_auto_realm(cx, &*global);
                ready_promise.append_native_handler(&mut realm.current_realm(), &handler);
            }));

        // Step 2. Return promise. Complete the rest of these steps asynchronously.
        load_promise
    }

    /// <https://html.spec.whatwg.org/multipage/#customstateset>
    fn Size(&self, cx: &mut JSContext) -> u32 {
        self.size(cx)
    }
}

impl Setlike for FontFaceSet {
    type Key = DomRoot<FontFace>;

    #[inline(always)]
    fn get_index(&self, cx: &mut JSContext, index: u32) -> Option<Self::Key> {
        self.flush_author_font_set(cx);
        self.set_entries
            .borrow()
            .get(index as usize)
            .map(|face| face.as_rooted())
    }

    #[inline(always)]
    fn size(&self, cx: &mut JSContext) -> u32 {
        self.flush_author_font_set(cx);
        self.set_entries.borrow().len() as u32
    }

    #[inline(always)]
    fn add(&self, _cx: &mut JSContext, face: Self::Key) {
        self.set_entries.borrow_mut().push(face.as_traced());
    }

    #[inline(always)]
    fn has(&self, cx: &mut JSContext, target: Self::Key) -> bool {
        self.flush_author_font_set(cx);
        self.contains_face(&target)
    }

    #[inline(always)]
    fn clear(&self, cx: &mut JSContext) {
        self.flush_author_font_set(cx);
        self.set_entries.borrow_mut().clear();
    }

    #[inline(always)]
    fn delete(&self, cx: &mut JSContext, to_delete: Self::Key) -> bool {
        self.flush_author_font_set(cx);
        self.delete_face(&to_delete)
    }
}

/// Represents a parsed query for [`FontFaceSet::load`] and [`FontFaceSet::check`].
///
/// [`FontFaceSet::load`]: https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-load
/// [`FontFaceSet::check`]: https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-check
struct FontQueryParameters {
    families: FontFamilyList,
    // TODO: Store a font descriptor here once we actually use that for matching.
}

/// Returned from <https://drafts.csswg.org/css-font-loading/#find-the-matching-font-faces> to indicate failure.
struct FontQuerySyntaxError;

impl FontQueryParameters {
    /// Implements Steps 1 and 3 of <https://drafts.csswg.org/css-font-loading/#find-the-matching-font-faces>.
    fn parse(document: &Document, font: &str) -> Result<Self, FontQuerySyntaxError> {
        // Step 1. Parse font using the CSS value syntax of the font property.
        // If a syntax error occurs, return a syntax error.
        // If the parsed value is a CSS-wide keyword, return a syntax error.
        // Absolutize all relative lengths against the initial values of the corresponding properties.
        // (For example, a relative font weight like bolder is evaluated against the initial value normal.)
        // Step 3. Let font family list be the list of font families parsed from font,
        // and font style be the other font style attributes parsed from font.
        let font_family;

        let urlextradata = document.url().into_url().into();
        let parser_context = parser_context_for_document(
            document,
            CssRuleType::FontFace,
            ParsingMode::DEFAULT,
            &urlextradata,
        );

        let mut input = ParserInput::new(font);
        let mut parser = Parser::new(&mut input);
        let Ok(font_shorthand) =
            parser.parse_entirely(|parser| font::parse_value(&parser_context, parser))
        else {
            return Err(FontQuerySyntaxError);
        };

        match font_shorthand.font_family {
            specified_font::FontFamily::Values(family_list) => font_family = family_list,
            specified_font::FontFamily::System(_) => return Err(FontQuerySyntaxError),
        }

        Ok(Self {
            families: font_family,
        })
    }
}

fn any_character_in_any_unicode_range(text: &str, unicode_ranges: &[UnicodeRange]) -> bool {
    for character in text.chars() {
        for unicode_range in unicode_ranges {
            if (unicode_range.start..=unicode_range.end).contains(&(character as u32)) {
                return true;
            }
        }
    }
    false
}
