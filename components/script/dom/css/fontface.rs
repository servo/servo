/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use fonts::{
    FontContext, FontContextWebFontMethods, FontFaceRuleWithOrigin, FontTemplate,
    LowercaseFontFamilyName,
};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use style::error_reporting::ParseErrorReporter;
use style::font_face::SourceList;
use style::properties::font_face::Descriptors;
use style::shared_lock::StylesheetGuards;
use style::stylesheets::{CssRuleType, FontFaceRule, UrlExtraData};
use style_traits::{ParsingMode, ToCss};

use crate::css::parser_context_for_document_with_reporter;
use crate::dom::bindings::buffer_source::get_buffer_source_copy;
use crate::dom::bindings::codegen::Bindings::FontFaceBinding::{
    FontFaceDescriptors, FontFaceLoadStatus, FontFaceMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes;
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, StringOrArrayBufferViewOrArrayBuffer,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::css::fontfaceset::FontFaceSet;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::NodeTraits;
use crate::dom::promise::Promise;
use crate::dom::window::Window;

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
    #[conditional_malloc_size_of]
    font_status_promise: Rc<Promise>,

    /// The `@font-face` rule that this `FontFace` object is [css-connected] to, if any.
    ///
    /// [css-connected]: https://drafts.csswg.org/css-font-loading/#css-connected
    #[no_trace]
    css_font_face_rule: DomRefCell<Option<FontFaceRuleWithOrigin>>,
}

/// Given the various font face descriptors, construct the equivalent `@font-face` css rule as a
/// string and parse it using `style` crate. Returns `Err(Error::Syntax)` if parsing fails.
///
/// Due to lack of support in the `style` crate, parsing the whole `@font-face` rule is much easier
/// to implement than parsing each declaration on its own.
fn parse_font_face_descriptors(
    global: &GlobalScope,
    family_name: &DOMString,
    sources: Option<&DOMString>,
    input_descriptors: &FontFaceDescriptors,
) -> Fallible<FontFaceRule> {
    let window = global.as_window(); // TODO: Support calling FontFace APIs from Worker
    let document = window.Document();
    let url_data = UrlExtraData(document.owner_global().api_base_url().get_arc());
    let error_reporter = FontFaceErrorReporter {
        not_encountered_error: Cell::new(true),
    };
    let parser_context = parser_context_for_document_with_reporter(
        &document,
        CssRuleType::FontFace,
        ParsingMode::DEFAULT,
        &url_data,
        &error_reporter,
    );

    let FontFaceDescriptors {
        ascentOverride,
        descentOverride,
        display,
        featureSettings,
        lineGapOverride,
        stretch,
        style,
        unicodeRange,
        variationSettings,
        weight,
    } = input_descriptors;

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
        font-variation-settings: {variationSettings};
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

    if let Some(ref mut sources) = parsed_font_face_rule.descriptors.src {
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
        Err(Error::Syntax(None))
    }
}

/// Converts the descriptors of a `@font-face` rule (as defined by stylo) to
/// the the IDL `FontFaceDescriptors` dictionary used by the JS interface.
fn serialize_parsed_descriptors(descriptors: &Descriptors) -> FontFaceDescriptors {
    FontFaceDescriptors {
        ascentOverride: descriptors.ascent_override.to_css_string().into(),
        descentOverride: descriptors.descent_override.to_css_string().into(),
        display: descriptors.font_display.to_css_string().into(),
        featureSettings: descriptors.font_feature_settings.to_css_string().into(),
        lineGapOverride: descriptors.line_gap_override.to_css_string().into(),
        stretch: descriptors.font_stretch.to_css_string().into(),
        style: descriptors.font_style.to_css_string().into(),
        unicodeRange: descriptors.unicode_range.to_css_string().into(),
        variationSettings: descriptors.font_variation_settings.to_css_string().into(),
        weight: descriptors.font_weight.to_css_string().into(),
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
    fn new_failed_font_face(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        let font_status_promise = Promise::new(cx, global);
        // If any of them fail to parse correctly, reject font face’s [[FontStatusPromise]] with a
        // DOMException named "SyntaxError"
        font_status_promise.reject_error(cx, Error::Syntax(None));

        // set font face’s corresponding attributes to the empty string, and set font face’s status
        // attribute to "error"
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self {
                reflector: Reflector::new(),
                font_face_set: MutNullableDom::default(),
                font_status_promise,
                family_name: DomRefCell::default(),
                urls: Default::default(),
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
                css_font_face_rule: Default::default(),
            }),
            global,
            proto,
        )
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    fn new_inherited(
        family_name: DOMString,
        urls: Option<SourceList>,
        descriptors: &Descriptors,
        font_status_promise: Rc<Promise>,
    ) -> Self {
        Self {
            reflector: Reflector::new(),

            // Set font face’s status attribute to "unloaded".
            status: Cell::new(FontFaceLoadStatus::Unloaded),

            // Set font face’s corresponding attributes to the serialization of the parsed values.
            descriptors: DomRefCell::new(serialize_parsed_descriptors(descriptors)),

            font_face_set: MutNullableDom::default(),
            family_name: DomRefCell::new(family_name),
            urls: DomRefCell::new(urls),
            template: RefCell::default(),
            font_status_promise,
            css_font_face_rule: Default::default(),
        }
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        font_family: DOMString,
        urls: Option<SourceList>,
        descriptors: &Descriptors,
        font_status_promise: Rc<Promise>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self::new_inherited(
                font_family,
                urls,
                descriptors,
                font_status_promise,
            )),
            global,
            proto,
        )
    }

    /// Constructs a unrooted `FontFace` object for a font that is backed by a `@font-face` rule.
    pub(crate) fn new_inherited_for_web_font(
        family_name: DOMString,
        descriptors: FontFaceDescriptors,
        src: Option<SourceList>,
        font_status_promise: Rc<Promise>,
        font_face_rule: FontFaceRuleWithOrigin,
    ) -> Self {
        Self {
            reflector: Reflector::new(),
            status: Cell::new(FontFaceLoadStatus::Loading),
            descriptors: DomRefCell::new(descriptors),
            font_face_set: MutNullableDom::default(),
            family_name: DomRefCell::new(family_name),
            urls: DomRefCell::new(src),
            template: RefCell::default(),
            font_status_promise,
            css_font_face_rule: DomRefCell::new(Some(font_face_rule)),
        }
    }

    /// Constructs a `FontFace` object for a font that is backed by a `@font-face` rule.
    pub(crate) fn new_for_web_font(
        cx: &mut JSContext,
        global: &GlobalScope,
        font_face_rule: FontFaceRuleWithOrigin,
        guards: &StylesheetGuards,
    ) -> Option<DomRoot<Self>> {
        let new_web_font_ref = font_face_rule.read_with(guards);
        let Some(family_name) = new_web_font_ref
            .descriptors
            .font_family
            .as_ref()
            .map(|name| DOMString::from(&*name.name))
        else {
            // Web fonts without a family name are not loaded, and they should not appear in document.fonts either.
            return None;
        };

        // https://drafts.csswg.org/css-font-loading/#font-face-css-connection
        // > The FontFace object corresponding to a @font-face rule has its family, style, weight, stretch,
        // > unicodeRange, variant, and featureSettings attributes set to the same value as the corresponding
        // > descriptors in the @font-face rule.
        let descriptors = serialize_parsed_descriptors(&new_web_font_ref.descriptors);

        let font_status_promise = Promise::new(cx, global);
        Some(reflect_dom_object_with_proto(
            cx,
            Box::new(Self::new_inherited_for_web_font(
                family_name,
                descriptors,
                new_web_font_ref.descriptors.src.clone(),
                font_status_promise,
                font_face_rule,
            )),
            global,
            None,
        ))
    }

    /// Mark this font face as *not* [css-connected].
    ///
    /// [css-connected]: https://drafts.csswg.org/css-font-loading/#css-connected
    pub(crate) fn disconnect_from_css(&self) {
        *self.css_font_face_rule.borrow_mut() = None;
    }

    /// Return true if the `FontFace` is [css-connected] *and* was created by the provided
    /// `@font-face` rule.
    ///
    /// [css-connected]: https://drafts.csswg.org/css-font-loading/#css-connected
    pub(crate) fn is_connected_to_font_face_rule(
        &self,
        target_rule: &FontFaceRuleWithOrigin,
    ) -> bool {
        self.css_font_face_rule
            .borrow()
            .as_ref()
            .is_some_and(|connected_rule| {
                FontFaceRuleWithOrigin::ptr_eq(connected_rule, target_rule)
            })
    }

    /// Step 3 of <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    fn load_from_data(&self, cx: &mut JSContext, global: &GlobalScope, data: Vec<u8>) {
        // Step 3.1 Set font face’s status attribute to "loading".
        self.status.set(FontFaceLoadStatus::Loading);

        // Step 3.2 For each FontFaceSet font face is in:
        if let Some(font_face_set) = self.font_face_set.get() {
            font_face_set.handle_font_face_status_changed(cx, self);
        }

        // Asynchronously, attempt to parse the data in it as a font. When this is completed,
        // successfully or not, queue a task to run the following steps synchronously:
        // FIXME: This is not asynchronous.
        let parsed_font_face_rule = self.font_face_rule(global);
        let result = parsed_font_face_rule
            .ok()
            .and_then(|parsed_font_face_rule| {
                global
                    .as_window()
                    .font_context()
                    .construct_web_font_from_data(&data, (&parsed_font_face_rule).into())
            });

        if let Some(template) = result {
            // Step 1. If the load was successful, font face now represents the parsed font; fulfill font face’s
            // [[FontStatusPromise]] with font face, and set its status attribute to "loaded".
            self.font_status_promise.resolve_native(cx, &self);
            self.status.set(FontFaceLoadStatus::Loaded);
            *self.template.borrow_mut() = Some(template);

            // For each FontFaceSet font face is in:
            if let Some(font_face_set) = self.font_face_set.get() {
                // Add font face to the FontFaceSet’s [[LoadedFonts]] list.
                // Remove font face from the FontFaceSet’s [[LoadingFonts]] list.
                // If font was the last item in that list (and so the list is now empty),
                // switch the FontFaceSet to loaded.
                font_face_set.handle_font_face_status_changed(cx, self);
            }
        } else {
            // Step 2. Otherwise, reject font face’s [[FontStatusPromise]] with a DOMException named "SyntaxError"
            // and set font face’s status attribute to "error".
            self.font_status_promise
                .reject_error(cx, Error::Syntax(None));
            self.status.set(FontFaceLoadStatus::Error);

            // For each FontFaceSet font face is in:
            if let Some(font_face_set) = self.font_face_set.get() {
                // Add font face to the FontFaceSet’s [[FailedFonts]] list.
                // Remove font face from the FontFaceSet’s [[LoadingFonts]] list.
                // If font was the last item in that list (and so the list is now empty),
                // switch the FontFaceSet to loaded.
                font_face_set.handle_font_face_status_changed(cx, self);
            }
        }
    }

    pub(super) fn set_associated_font_face_set(&self, font_face_set: &FontFaceSet) {
        self.font_face_set.set(Some(font_face_set));
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

        *self.descriptors.borrow_mut() =
            serialize_parsed_descriptors(&parsed_font_face_rule.descriptors);
        Ok(())
    }

    fn font_face_rule(&self, global: &GlobalScope) -> Fallible<FontFaceRule> {
        // TODO: We should not have to parse the descriptors over and over again here.
        // We can probably store them on the `FontFace` instead.
        parse_font_face_descriptors(
            global,
            &self.family_name.borrow(),
            None,
            &self.descriptors.borrow(),
        )
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
    fn Load(&self, cx: &mut JSContext) -> Rc<Promise> {
        // Step 2. If font face’s [[Urls]] slot is null, or its status attribute is anything
        // other than "unloaded", return font face’s [[FontStatusPromise]] and abort these
        // steps.
        let Some(sources) = self.urls.borrow_mut().take() else {
            return self.font_status_promise.clone();
        };
        if self.status.get() != FontFaceLoadStatus::Unloaded {
            return self.font_status_promise.clone();
        }

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
                task_source.queue(task!(resolve_font_face_load_task: move |cx| {
                    let font_face = trusted.root();

                    match load_result {
                        None => {
                            // Step 5.1. If the attempt to load fails, reject font face’s
                            // [[FontStatusPromise]] with a DOMException whose name is "NetworkError"
                            // and set font face’s status attribute to "error".
                            font_face.status.set(FontFaceLoadStatus::Error);
                            font_face.font_status_promise.reject_error(cx, Error::Network(None));
                        }
                        Some(template) => {
                            // Step 5.2. Otherwise, font face now represents the loaded font;
                            // fulfill font face’s [[FontStatusPromise]] with font face and set
                            // font face’s status attribute to "loaded".
                            font_face.status.set(FontFaceLoadStatus::Loaded);
                            let old_template = font_face.template.borrow_mut().replace((family_name, template));
                            debug_assert!(old_template.is_none(), "FontFace's template must be intialized only once");
                            font_face.font_status_promise.resolve_native(cx, &font_face);
                        }
                    }

                    if let Some(font_face_set) = font_face.font_face_set.get() {
                        // For each FontFaceSet font face is in: ...
                        //
                        // This implements steps 5.1.1, 5.1.2, 5.2.1 and 5.2.2 - these
                        // take care of changing the status of the `FontFaceSet` in which this
                        // `FontFace` is a member, for both failed and successful load.
                        font_face_set.handle_font_face_status_changed(cx, &font_face);
                    }
                }));
            },
        );

        // We parse the descriptors again because they are stored as `DOMString`s in this `FontFace`
        // but the `load_web_font_for_script` API needs parsed values.
        let parsed_font_face_rule = self
            .font_face_rule(&global)
            .expect("Parsing shouldn't fail as descriptors are valid by construction");

        // Construct a WebFontDocumentContext object for the current document.
        let document_context = global.as_window().web_font_context(cx.no_gc());

        // Step 4. Using the value of font face’s [[Urls]] slot, attempt to load a font as defined
        // in [CSS-FONTS-3], as if it was the value of a @font-face rule’s src descriptor.
        // TODO: FontFaceSet is not supported on Workers yet. The `as_window` call below should be
        // replaced when we do support it.
        global.as_window().font_context().load_web_font_for_script(
            global.webview_id(),
            sources,
            (&parsed_font_face_rule).into(),
            finished_callback,
            &document_context,
        );

        // Step 3. Set font face’s status attribute to "loading", return font face’s
        // [[FontStatusPromise]], and continue executing the rest of this algorithm asynchronously.
        self.status.set(FontFaceLoadStatus::Loading);

        // See <https://github.com/w3c/csswg-drafts/issues/13235>:
        // All browsers switch the FontFaceSet to loading, but this is currently missing
        // from the specification.
        if let Some(font_face_set) = self.font_face_set.get() {
            font_face_set.handle_font_face_status_changed(cx, self);
        }

        self.font_status_promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#dom-fontface-loaded>
    fn Loaded(&self) -> Rc<Promise> {
        self.font_status_promise.clone()
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        family: DOMString,
        source: UnionTypes::StringOrArrayBufferViewOrArrayBuffer,
        descriptors: &FontFaceDescriptors,
    ) -> DomRoot<FontFace> {
        // Step 2. If the source argument was a CSSOMString, set font face’s internal [[Urls]] slot to the string.
        let url_source = if let StringOrArrayBufferViewOrArrayBuffer::String(source) = &source {
            Some(source)
        } else {
            None
        };
        // All the rest of the comments are part of step 1:

        // Parse the family argument, and the members of the descriptors argument,
        // according to the grammars of the corresponding descriptors of the CSS @font-face rule If
        // the source argument is a CSSOMString, parse it according to the grammar of the CSS src
        // descriptor of the @font-face rule.
        let global = window.as_global_scope();
        let parse_result = parse_font_face_descriptors(global, &family, url_source, descriptors);

        let Ok(ref parsed_font_face_rule) = parse_result else {
            // If any of them fail to parse correctly, reject font face’s
            // [[FontStatusPromise]] with a DOMException named "SyntaxError", set font face’s
            // corresponding attributes to the empty string, and set font face’s status attribute
            // to "error".
            return Self::new_failed_font_face(cx, global, proto);
        };

        // Set its internal [[FontStatusPromise]] slot to a fresh pending Promise object.
        let font_status_promise = Promise::new(cx, global);

        let sources = parsed_font_face_rule.descriptors.src.clone();
        // Let font face be a fresh FontFace object.
        let font_face = FontFace::new(
            cx,
            global,
            proto,
            family,
            sources,
            &parsed_font_face_rule.descriptors,
            font_status_promise,
        );

        // If font face’s status is "error", terminate this algorithm;
        // otherwise, complete the rest of these steps asynchronously.
        if font_face.Status() == FontFaceLoadStatus::Error {
            return font_face;
        }

        // Step 2. If the source argument was a BufferSource, set font face’s internal
        // [[Data]] slot to the passed argument.
        // Step 3. If font face’s [[Data]] slot is not null, queue a task to run the following steps
        // synchronously:
        let font_face_bytes = match source {
            StringOrArrayBufferViewOrArrayBuffer::String(_) => {
                // Return font face.
                return font_face;
            },
            StringOrArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => {
                get_buffer_source_copy(&ArrayBufferViewOrArrayBuffer::ArrayBufferView(view))
            },
            StringOrArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => {
                get_buffer_source_copy(&ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer))
            },
        };
        let trusted_font_face = Trusted::new(&*font_face);
        let trusted_global = Trusted::new(global);
        global
            .task_manager()
            .font_loading_task_source()
            .queue(task!(
                load_font_from_arraybuffer: move |cx| {
                    let font_face = trusted_font_face.root();
                    let global = trusted_global.root();

                    font_face.load_from_data(cx, &global, font_face_bytes);
                }
            ));

        // Return font face.
        font_face
    }
}
