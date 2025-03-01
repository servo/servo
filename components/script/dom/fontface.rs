/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use fonts::{FontContext, FontContextWebFontMethods, FontTemplate, LowercaseFontFamilyName};
use js::rust::HandleObject;
use style::error_reporting::ParseErrorReporter;
use style::font_face::SourceList;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, FontFaceRule, Origin, UrlExtraData};
use style_traits::{ParsingMode, ToCss};

use super::bindings::cell::DomRefCell;
use super::bindings::codegen::UnionTypes::StringOrArrayBufferViewOrArrayBuffer;
use super::bindings::error::{Error, ErrorResult, Fallible};
use super::bindings::refcounted::Trusted;
use super::bindings::reflector::DomGlobal;
use super::bindings::root::MutNullableDom;
use super::types::FontFaceSet;
use crate::dom::bindings::codegen::Bindings::FontFaceBinding::{
    FontFaceDescriptors, FontFaceLoadStatus, FontFaceMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://drafts.csswg.org/css-font-loading/#fontface-interface>
#[dom_struct]
pub struct FontFace {
    reflector: Reflector,
    status: Cell<FontFaceLoadStatus>,
    family_name: DomRefCell<DOMString>,
    descriptors: DomRefCell<FontFaceDescriptors>,

    /// A reference to the [`FontFaceSet`] that this `FontFace` is a member of, if it has been
    /// added to one. `None` otherwise. The spec suggests that a `FontFace` can be a member of
    /// multiple `FontFaceSet`s, but this doesn't seem to be the case in practice, as the
    /// `FontFaceSet` constructor is not exposed on the global scope.
    font_face_set: MutNullableDom<FontFaceSet>,

    /// This holds the [`FontTemplate`] resulting from loading this `FontFace`, to be used when the
    /// `FontFace` is added to the global `FontFaceSet` and thus the `[FontContext]`.
    //
    // TODO: This could potentially share the `FontTemplateRef` created by `FontContext`, rather
    // than having its own copy of the template.
    #[no_trace = "Does not contain managed objects"]
    template: RefCell<Option<(LowercaseFontFamilyName, FontTemplate)>>,

    #[no_trace = "Does not contain managed objects"]
    /// <https://drafts.csswg.org/css-font-loading/#m-fontface-urls-slot>
    urls: DomRefCell<Option<SourceList>>,

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-fontstatuspromise-slot>
    #[ignore_malloc_size_of = "Rc"]
    font_status_promise: Rc<Promise>,
}

/// Given the various font face descriptors, construct the equivalent `@font-face` css rule as a
/// string and parse it using `style` crate. Returns `Err(Error::Syntax)` if parsing fails.
///
/// Due to lack of support in the `style` crate, parsing the whole `@font-face` rule is much easier
/// to implement than parsing each declaration on its own.
fn parse_font_face_descriptors(
    global: &GlobalScope,
    family_name: &DOMString,
    sources: Option<&str>,
    input_descriptors: &FontFaceDescriptors,
) -> Fallible<FontFaceRule> {
    let window = global.as_window(); // TODO: Support calling FontFace APIs from Worker
    let quirks_mode = window.Document().quirks_mode();
    let url_data = UrlExtraData(window.get_url().get_arc());
    let error_reporter = FontFaceErrorReporter {
        not_encountered_error: Cell::new(true),
    };
    let parser_context = ParserContext::new(
        Origin::Author,
        &url_data,
        Some(CssRuleType::FontFace),
        ParsingMode::DEFAULT,
        quirks_mode,
        /* namespaces = */ Default::default(),
        Some(&error_reporter as &dyn ParseErrorReporter),
        None,
    );

    let FontFaceDescriptors {
        ref ascentOverride,
        ref descentOverride,
        ref display,
        ref featureSettings,
        ref lineGapOverride,
        ref stretch,
        ref style,
        ref unicodeRange,
        ref variationSettings,
        ref weight,
    } = input_descriptors;

    let _ = variationSettings; // TODO: Stylo doesn't parse font-variation-settings yet.
    let maybe_sources = sources.map_or_else(String::new, |sources| format!("src: {sources};"));
    let font_face_rule = format!(
        r"
        ascent-override: {ascentOverride};
        descent-override: {descentOverride};
        font-display: {display};
        font-family: {family_name};
        font-feature-settings: {featureSettings};
        font-stretch: {stretch};
        font-style: {style};
        font-weight: {weight};
        line-gap-override: {lineGapOverride};
        unicode-range: {unicodeRange};
        {maybe_sources}
    "
    );

    // TODO: Should this be the source location in the script that invoked the font face API?
    let location = cssparser::SourceLocation { line: 0, column: 0 };
    let mut input = ParserInput::new(&font_face_rule);
    let mut parser = Parser::new(&mut input);
    let mut parsed_font_face_rule =
        style::font_face::parse_font_face_block(&parser_context, &mut parser, location);

    if let Some(ref mut sources) = parsed_font_face_rule.sources {
        let supported_sources: Vec<_> = sources
            .0
            .iter()
            .rev()
            .filter(FontContext::is_supported_web_font_source)
            .cloned()
            .collect();
        if supported_sources.is_empty() {
            error_reporter.not_encountered_error.set(false);
        } else {
            sources.0 = supported_sources;
        }
    }

    if error_reporter.not_encountered_error.get() {
        Ok(parsed_font_face_rule)
    } else {
        Err(Error::Syntax)
    }
}

fn serialize_parsed_descriptors(font_face_rule: &FontFaceRule) -> FontFaceDescriptors {
    FontFaceDescriptors {
        ascentOverride: font_face_rule.ascent_override.to_css_string().into(),
        descentOverride: font_face_rule.descent_override.to_css_string().into(),
        display: font_face_rule.display.to_css_string().into(),
        featureSettings: font_face_rule.feature_settings.to_css_string().into(),
        lineGapOverride: font_face_rule.line_gap_override.to_css_string().into(),
        stretch: font_face_rule.stretch.to_css_string().into(),
        style: font_face_rule.style.to_css_string().into(),
        unicodeRange: font_face_rule.unicode_range.to_css_string().into(),
        variationSettings: font_face_rule.variation_settings.to_css_string().into(),
        weight: font_face_rule.weight.to_css_string().into(),
    }
}

struct FontFaceErrorReporter {
    not_encountered_error: Cell<bool>,
}

impl ParseErrorReporter for FontFaceErrorReporter {
    fn report_error(
        &self,
        _url: &UrlExtraData,
        _location: cssparser::SourceLocation,
        _error: style::error_reporting::ContextualParseError,
    ) {
        self.not_encountered_error.set(false);
    }
}

impl FontFace {
    /// Construct a [`FontFace`] to be used in the case of failure in parsing the
    /// font face descriptors.
    fn new_failed_font_face(global: &GlobalScope, can_gc: CanGc) -> Self {
        let font_status_promise = Promise::new(global, can_gc);
        // If any of them fail to parse correctly, reject font face’s [[FontStatusPromise]] with a
        // DOMException named "SyntaxError"
        font_status_promise.reject_error(Error::Syntax, can_gc);

        // set font face’s corresponding attributes to the empty string, and set font face’s status
        // attribute to "error"
        Self {
            reflector: Reflector::new(),
            font_face_set: MutNullableDom::default(),
            font_status_promise,
            family_name: DomRefCell::default(),
            urls: DomRefCell::default(),
            descriptors: DomRefCell::new(FontFaceDescriptors {
                ascentOverride: DOMString::new(),
                descentOverride: DOMString::new(),
                display: DOMString::new(),
                featureSettings: DOMString::new(),
                lineGapOverride: DOMString::new(),
                stretch: DOMString::new(),
                style: DOMString::new(),
                unicodeRange: DOMString::new(),
                variationSettings: DOMString::new(),
                weight: DOMString::new(),
            }),
            status: Cell::new(FontFaceLoadStatus::Error),
            template: RefCell::default(),
        }
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    fn new_inherited(
        global: &GlobalScope,
        family_name: DOMString,
        source: StringOrArrayBufferViewOrArrayBuffer,
        descriptors: &FontFaceDescriptors,
        can_gc: CanGc,
    ) -> Self {
        // TODO: Add support for ArrayBuffer and ArrayBufferView sources.
        let StringOrArrayBufferViewOrArrayBuffer::String(ref source_string) = source else {
            return Self::new_failed_font_face(global, can_gc);
        };

        // Step 1. Parse the family argument, and the members of the descriptors argument,
        // according to the grammars of the corresponding descriptors of the CSS @font-face rule If
        // the source argument is a CSSOMString, parse it according to the grammar of the CSS src
        // descriptor of the @font-face rule.
        let parse_result =
            parse_font_face_descriptors(global, &family_name, Some(source_string), descriptors);

        let Ok(ref parsed_font_face_rule) = parse_result else {
            // If any of them fail to parse correctly, reject font face’s
            // [[FontStatusPromise]] with a DOMException named "SyntaxError", set font face’s
            // corresponding attributes to the empty string, and set font face’s status attribute
            // to "error".
            return Self::new_failed_font_face(global, can_gc);
        };

        // Set its internal [[FontStatusPromise]] slot to a fresh pending Promise object.
        let font_status_promise = Promise::new(global, can_gc);

        let sources = parsed_font_face_rule
            .sources
            .clone()
            .expect("Sources should be non-None after validation");

        // Let font face be a fresh FontFace object.
        Self {
            reflector: Reflector::new(),

            // Set font face’s status attribute to "unloaded".
            status: Cell::new(FontFaceLoadStatus::Unloaded),

            // Set font face’s corresponding attributes to the serialization of the parsed values.
            descriptors: DomRefCell::new(serialize_parsed_descriptors(parsed_font_face_rule)),

            font_face_set: MutNullableDom::default(),
            family_name: DomRefCell::new(family_name.clone()),
            urls: DomRefCell::new(Some(sources)),
            template: RefCell::default(),
            font_status_promise,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        font_family: DOMString,
        source: StringOrArrayBufferViewOrArrayBuffer,
        descriptors: &FontFaceDescriptors,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(
                global,
                font_family,
                source,
                descriptors,
                can_gc,
            )),
            global,
            proto,
            can_gc,
        )
    }

    pub(super) fn set_associated_font_face_set(&self, font_face_set: &FontFaceSet) {
        self.font_face_set.set(Some(font_face_set));
    }

    pub(super) fn loaded(&self) -> bool {
        self.status.get() == FontFaceLoadStatus::Loaded
    }

    pub(super) fn template(&self) -> Option<(LowercaseFontFamilyName, FontTemplate)> {
        self.template.borrow().clone()
    }

    /// Implements the body of the setter for the descriptor attributes of the [`FontFace`] interface.
    ///
    /// <https://drafts.csswg.org/css-font-loading/#fontface-interface>:
    /// On setting, parse the string according to the grammar for the corresponding @font-face
    /// descriptor. If it does not match the grammar, throw a SyntaxError; otherwise, set the attribute
    /// to the serialization of the parsed value.
    fn validate_and_set_descriptors(&self, new_descriptors: FontFaceDescriptors) -> ErrorResult {
        let global = self.global();
        let parsed_font_face_rule = parse_font_face_descriptors(
            &global,
            &self.family_name.borrow(),
            None,
            &new_descriptors,
        )?;

        *self.descriptors.borrow_mut() = serialize_parsed_descriptors(&parsed_font_face_rule);
        Ok(())
    }
}

impl FontFaceMethods<crate::DomTypeHolder> for FontFace {
    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-family>
    fn Family(&self) -> DOMString {
        self.family_name.borrow().clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-family>
    fn SetFamily(&self, family_name: DOMString) -> ErrorResult {
        let descriptors = self.descriptors.borrow();
        let global = self.global();
        let _ = parse_font_face_descriptors(&global, &family_name, None, &descriptors)?;
        *self.family_name.borrow_mut() = family_name;
        Ok(())
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-style>
    fn Style(&self) -> DOMString {
        self.descriptors.borrow().style.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-style>
    fn SetStyle(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.style = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-weight>
    fn Weight(&self) -> DOMString {
        self.descriptors.borrow().weight.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-weight>
    fn SetWeight(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.weight = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-stretch>
    fn Stretch(&self) -> DOMString {
        self.descriptors.borrow().stretch.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-stretch>
    fn SetStretch(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.stretch = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-unicoderange>
    fn UnicodeRange(&self) -> DOMString {
        self.descriptors.borrow().unicodeRange.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-unicoderange>
    fn SetUnicodeRange(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.unicodeRange = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-featuresettings>
    fn FeatureSettings(&self) -> DOMString {
        self.descriptors.borrow().featureSettings.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-featuresettings>
    fn SetFeatureSettings(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.featureSettings = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-variationsettings>
    fn VariationSettings(&self) -> DOMString {
        self.descriptors.borrow().variationSettings.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-variationsettings>
    fn SetVariationSettings(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.variationSettings = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-display>
    fn Display(&self) -> DOMString {
        self.descriptors.borrow().display.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-display>
    fn SetDisplay(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.display = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-ascentoverride>
    fn AscentOverride(&self) -> DOMString {
        self.descriptors.borrow().ascentOverride.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-ascentoverride>
    fn SetAscentOverride(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.ascentOverride = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-descentoverride>
    fn DescentOverride(&self) -> DOMString {
        self.descriptors.borrow().descentOverride.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-descentoverride>
    fn SetDescentOverride(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.descentOverride = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-linegapoverride>
    fn LineGapOverride(&self) -> DOMString {
        self.descriptors.borrow().lineGapOverride.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-linegapoverride>
    fn SetLineGapOverride(&self, value: DOMString) -> ErrorResult {
        let mut new_descriptors = self.descriptors.borrow().clone();
        new_descriptors.lineGapOverride = value;
        self.validate_and_set_descriptors(new_descriptors)
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-status>
    fn Status(&self) -> FontFaceLoadStatus {
        self.status.get()
    }

    /// The load() method of FontFace forces a url-based font face to request its font data and
    /// load. For fonts constructed from a buffer source, or fonts that are already loading or
    /// loaded, it does nothing.
    /// <https://drafts.csswg.org/css-font-loading/#font-face-load>
    fn Load(&self) -> Rc<Promise> {
        let Some(sources) = self.urls.borrow_mut().take() else {
            // Step 2. If font face’s [[Urls]] slot is null, or its status attribute is anything
            // other than "unloaded", return font face’s [[FontStatusPromise]] and abort these
            // steps.
            return self.font_status_promise.clone();
        };

        // FontFace must not be loaded at this point as `self.urls` is not None, implying `Load`
        // wasn't called already. In our implementation, `urls` is set after parsing, so it
        // cannot be `Some` if the status is `Error`.
        debug_assert_eq!(self.status.get(), FontFaceLoadStatus::Unloaded);

        let global = self.global();
        let trusted = Trusted::new(self);
        let task_source = global
            .task_manager()
            .font_loading_task_source()
            .to_sendable();

        let finished_callback = Box::new(
            move |family_name: LowercaseFontFamilyName, load_result: Option<_>| {
                let trusted = trusted.clone();

                // Step 5. When the load operation completes, successfully or not, queue a task to
                // run the following steps synchronously:
                task_source.queue(task!(resolve_font_face_load_task: move || {
                    let font_face = trusted.root();

                    match load_result {
                        None => {
                            // Step 5.1. If the attempt to load fails, reject font face’s
                            // [[FontStatusPromise]] with a DOMException whose name is "NetworkError"
                            // and set font face’s status attribute to "error".
                            font_face.status.set(FontFaceLoadStatus::Error);
                            font_face.font_status_promise.reject_error(Error::Network, CanGc::note());
                        }
                        Some(template) => {
                            // Step 5.2. Otherwise, font face now represents the loaded font;
                            // fulfill font face’s [[FontStatusPromise]] with font face and set
                            // font face’s status attribute to "loaded".
                            font_face.status.set(FontFaceLoadStatus::Loaded);
                            let old_template = font_face.template.borrow_mut().replace((family_name, template));
                            debug_assert!(old_template.is_none(), "FontFace's template must be intialized only once");
                            font_face.font_status_promise.resolve_native(&font_face, CanGc::note());
                        }
                    }

                    if let Some(font_face_set) = font_face.font_face_set.get() {
                        // For each FontFaceSet font face is in: ...
                        //
                        // This implements steps 5.1.1, 5.1.2, 5.2.1 and 5.2.2 - these
                        // take care of changing the status of the `FontFaceSet` in which this
                        // `FontFace` is a member, for both failed and successful load.
                        font_face_set.handle_font_face_status_changed(&font_face);
                    }
                }));
            },
        );

        // We parse the descriptors again because they are stored as `DOMString`s in this `FontFace`
        // but the `load_web_font_for_script` API needs parsed values.
        let parsed_font_face_rule = parse_font_face_descriptors(
            &global,
            &self.family_name.borrow(),
            None,
            &self.descriptors.borrow(),
        )
        .expect("Parsing shouldn't fail as descriptors are valid by construction");

        // Step 4. Using the value of font face’s [[Urls]] slot, attempt to load a font as defined
        // in [CSS-FONTS-3], as if it was the value of a @font-face rule’s src descriptor.
        // TODO: FontFaceSet is not supported on Workers yet. The `as_window` call below should be
        // replaced when we do support it.
        global.as_window().font_context().load_web_font_for_script(
            global.webview_id(),
            sources,
            (&parsed_font_face_rule).into(),
            finished_callback,
        );

        // Step 3. Set font face’s status attribute to "loading", return font face’s
        // [[FontStatusPromise]], and continue executing the rest of this algorithm asynchronously.
        self.status.set(FontFaceLoadStatus::Loading);
        self.font_status_promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-loaded>
    fn Loaded(&self) -> Rc<Promise> {
        self.font_status_promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        family: DOMString,
        source: UnionTypes::StringOrArrayBufferViewOrArrayBuffer,
        descriptors: &FontFaceDescriptors,
    ) -> DomRoot<FontFace> {
        let global = window.as_global_scope();
        FontFace::new(global, proto, family, source, descriptors, can_gc)
    }
}
