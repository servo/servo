/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::SanitizerBinding::{
    SanitizerAttribute, SanitizerAttributeNamespace, SanitizerConfig, SanitizerElementNamespace,
    SanitizerElementNamespaceWithAttributes, SanitizerElementWithAttributes, SanitizerMethods,
    SanitizerPresets,
};
use crate::dom::bindings::codegen::UnionTypes::SanitizerConfigOrSanitizerPresets;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

#[dom_struct]
pub(crate) struct Sanitizer {
    reflector_: Reflector,
    /// <https://wicg.github.io/sanitizer-api/#sanitizer-configuration>
    configuration: DomRefCell<SanitizerConfig>,
}

impl Sanitizer {
    fn new_inherited(configuration: SanitizerConfig) -> Sanitizer {
        Sanitizer {
            reflector_: Reflector::new(),
            configuration: DomRefCell::new(configuration),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        configuration: SanitizerConfig,
    ) -> DomRoot<Sanitizer> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(Sanitizer::new_inherited(configuration)),
            window,
            proto,
            cx,
        )
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-set-a-configuration>
    fn set_configuration(
        &self,
        configuration: SanitizerConfig,
        _allow_comments_and_data_attributes: bool,
    ) -> bool {
        // TODO:
        // Step 1. Canonicalize configuration with allowCommentsAndDataAttributes.

        // TODO:
        // Step 2. If configuration is not valid, then return false.

        // Step 3. Set sanitizer’s configuration to configuration.
        let mut sanitizer_configuration = self.configuration.borrow_mut();
        *sanitizer_configuration = configuration;

        // Step 4. Return true.
        true
    }
}

impl SanitizerMethods<crate::DomTypeHolder> for Sanitizer {
    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        configuration: SanitizerConfigOrSanitizerPresets,
    ) -> Fallible<DomRoot<Sanitizer>> {
        let configuration = match configuration {
            // Step 1. If configuration is a SanitizerPresets string, then:
            SanitizerConfigOrSanitizerPresets::SanitizerPresets(configuration) => {
                // Step 1.1. Assert: configuration is default.
                assert_eq!(configuration, SanitizerPresets::Default);

                // Step 1.2. Set configuration to the built-in safe default configuration.
                built_in_safe_default_configuration()
            },
            SanitizerConfigOrSanitizerPresets::SanitizerConfig(configuration) => configuration,
        };

        // Step 2. Let valid be the return value of set a configuration with configuration and true
        // on this.
        // Step 3. If valid is false, then throw a TypeError.
        let sanitizer = Sanitizer::new_with_proto(cx, window, proto, SanitizerConfig::default());
        if !sanitizer.set_configuration(configuration, true) {
            return Err(Error::Type(c"The configuration is invalid".into()));
        }

        Ok(sanitizer)
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-get>
    fn Get(&self) -> SanitizerConfig {
        // Step 1. Let config be this’s configuration.
        let config = self.configuration.borrow_mut();

        // TODO: Step 2 to Step 7

        // Step 8. Return config.
        (*config).clone()
    }
}

/// <https://wicg.github.io/sanitizer-api/#built-in-safe-default-configuration>
fn built_in_safe_default_configuration() -> SanitizerConfig {
    const ELEMENTS: &[(&str, &str, &[&str])] = &[
        ("math", MATHML_NAMESPACE, &[]),
        ("merror", MATHML_NAMESPACE, &[]),
        ("mfrac", MATHML_NAMESPACE, &[]),
        ("mi", MATHML_NAMESPACE, &[]),
        ("mmultiscripts", MATHML_NAMESPACE, &[]),
        ("mn", MATHML_NAMESPACE, &[]),
        (
            "mo",
            MATHML_NAMESPACE,
            &[
                "fence",
                "form",
                "largeop",
                "lspace",
                "maxsize",
                "minsize",
                "movablelimits",
                "rspace",
                "separator",
                "stretchy",
                "symmetric",
            ],
        ),
        ("mover", MATHML_NAMESPACE, &["accent"]),
        (
            "mpadded",
            MATHML_NAMESPACE,
            &["depth", "height", "lspace", "voffset", "width"],
        ),
        ("mphantom", MATHML_NAMESPACE, &[]),
        ("mprescripts", MATHML_NAMESPACE, &[]),
        ("mroot", MATHML_NAMESPACE, &[]),
        ("mrow", MATHML_NAMESPACE, &[]),
        ("ms", MATHML_NAMESPACE, &[]),
        ("mspace", MATHML_NAMESPACE, &["depth", "height", "width"]),
        ("msqrt", MATHML_NAMESPACE, &[]),
        ("mstyle", MATHML_NAMESPACE, &[]),
        ("msub", MATHML_NAMESPACE, &[]),
        ("msubsup", MATHML_NAMESPACE, &[]),
        ("msup", MATHML_NAMESPACE, &[]),
        ("mtable", MATHML_NAMESPACE, &[]),
        ("mtd", MATHML_NAMESPACE, &["columnspan", "rowspan"]),
        ("mtext", MATHML_NAMESPACE, &[]),
        ("mtr", MATHML_NAMESPACE, &[]),
        ("munder", MATHML_NAMESPACE, &["accentunder"]),
        ("munderover", MATHML_NAMESPACE, &["accent", "accentunder"]),
        ("semantics", MATHML_NAMESPACE, &[]),
        ("a", HTML_NAMESPACE, &["href", "hreflang", "type"]),
        ("abbr", HTML_NAMESPACE, &[]),
        ("address", HTML_NAMESPACE, &[]),
        ("article", HTML_NAMESPACE, &[]),
        ("aside", HTML_NAMESPACE, &[]),
        ("b", HTML_NAMESPACE, &[]),
        ("bdi", HTML_NAMESPACE, &[]),
        ("bdo", HTML_NAMESPACE, &[]),
        ("blockquote", HTML_NAMESPACE, &["cite"]),
        ("body", HTML_NAMESPACE, &[]),
        ("br", HTML_NAMESPACE, &[]),
        ("caption", HTML_NAMESPACE, &[]),
        ("cite", HTML_NAMESPACE, &[]),
        ("code", HTML_NAMESPACE, &[]),
        ("col", HTML_NAMESPACE, &["span"]),
        ("colgroup", HTML_NAMESPACE, &["span"]),
        ("data", HTML_NAMESPACE, &["value"]),
        ("dd", HTML_NAMESPACE, &[]),
        ("del", HTML_NAMESPACE, &["cite", "datetime"]),
        ("dfn", HTML_NAMESPACE, &[]),
        ("div", HTML_NAMESPACE, &[]),
        ("dl", HTML_NAMESPACE, &[]),
        ("dt", HTML_NAMESPACE, &[]),
        ("em", HTML_NAMESPACE, &[]),
        ("figcaption", HTML_NAMESPACE, &[]),
        ("figure", HTML_NAMESPACE, &[]),
        ("footer", HTML_NAMESPACE, &[]),
        ("h1", HTML_NAMESPACE, &[]),
        ("h2", HTML_NAMESPACE, &[]),
        ("h3", HTML_NAMESPACE, &[]),
        ("h4", HTML_NAMESPACE, &[]),
        ("h5", HTML_NAMESPACE, &[]),
        ("h6", HTML_NAMESPACE, &[]),
        ("head", HTML_NAMESPACE, &[]),
        ("header", HTML_NAMESPACE, &[]),
        ("hgroup", HTML_NAMESPACE, &[]),
        ("hr", HTML_NAMESPACE, &[]),
        ("html", HTML_NAMESPACE, &[]),
        ("i", HTML_NAMESPACE, &[]),
        ("ins", HTML_NAMESPACE, &["cite", "datetime"]),
        ("kbd", HTML_NAMESPACE, &[]),
        ("li", HTML_NAMESPACE, &["value"]),
        ("main", HTML_NAMESPACE, &[]),
        ("mark", HTML_NAMESPACE, &[]),
        ("menu", HTML_NAMESPACE, &[]),
        ("nav", HTML_NAMESPACE, &[]),
        ("ol", HTML_NAMESPACE, &["reversed", "start", "type"]),
        ("p", HTML_NAMESPACE, &[]),
        ("pre", HTML_NAMESPACE, &[]),
        ("q", HTML_NAMESPACE, &[]),
        ("rp", HTML_NAMESPACE, &[]),
        ("rt", HTML_NAMESPACE, &[]),
        ("ruby", HTML_NAMESPACE, &[]),
        ("s", HTML_NAMESPACE, &[]),
        ("samp", HTML_NAMESPACE, &[]),
        ("search", HTML_NAMESPACE, &[]),
        ("section", HTML_NAMESPACE, &[]),
        ("small", HTML_NAMESPACE, &[]),
        ("span", HTML_NAMESPACE, &[]),
        ("strong", HTML_NAMESPACE, &[]),
        ("sub", HTML_NAMESPACE, &[]),
        ("sup", HTML_NAMESPACE, &[]),
        ("table", HTML_NAMESPACE, &[]),
        ("tbody", HTML_NAMESPACE, &[]),
        ("td", HTML_NAMESPACE, &["colspan", "headers", "rowspan"]),
        ("tfoot", HTML_NAMESPACE, &[]),
        (
            "th",
            HTML_NAMESPACE,
            &["abbr", "colspan", "headers", "rowspan", "scope"],
        ),
        ("thead", HTML_NAMESPACE, &[]),
        ("time", HTML_NAMESPACE, &["datetime"]),
        ("title", HTML_NAMESPACE, &[]),
        ("tr", HTML_NAMESPACE, &[]),
        ("u", HTML_NAMESPACE, &[]),
        ("ul", HTML_NAMESPACE, &[]),
        ("var", HTML_NAMESPACE, &[]),
        ("wbr", HTML_NAMESPACE, &[]),
        ("a", SVG_NAMESPACE, &["href", "hreflang", "type"]),
        ("circle", SVG_NAMESPACE, &["cx", "cy", "pathLength", "r"]),
        ("defs", SVG_NAMESPACE, &[]),
        ("desc", SVG_NAMESPACE, &[]),
        (
            "ellipse",
            SVG_NAMESPACE,
            &["cx", "cy", "pathLength", "rx", "ry"],
        ),
        (
            "foreignObject",
            SVG_NAMESPACE,
            &["height", "width", "x", "y"],
        ),
        ("g", SVG_NAMESPACE, &[]),
        (
            "line",
            SVG_NAMESPACE,
            &["pathLength", "x1", "x2", "y1", "y2"],
        ),
        (
            "marker",
            SVG_NAMESPACE,
            &[
                "markerHeight",
                "markerUnits",
                "markerWidth",
                "orient",
                "preserveAspectRatio",
                "refX",
                "refY",
                "viewBox",
            ],
        ),
        ("metadata", SVG_NAMESPACE, &[]),
        ("path", SVG_NAMESPACE, &["d", "pathLength"]),
        ("polygon", SVG_NAMESPACE, &["pathLength", "points"]),
        ("polyline", SVG_NAMESPACE, &["pathLength", "points"]),
        (
            "rect",
            SVG_NAMESPACE,
            &["height", "pathLength", "rx", "ry", "width", "x", "y"],
        ),
        (
            "svg",
            SVG_NAMESPACE,
            &[
                "height",
                "preserveAspectRatio",
                "viewBox",
                "width",
                "x",
                "y",
            ],
        ),
        (
            "text",
            SVG_NAMESPACE,
            &["dx", "dy", "lengthAdjust", "rotate", "textLength", "x", "y"],
        ),
        (
            "textPath",
            SVG_NAMESPACE,
            &[
                "lengthAdjust",
                "method",
                "path",
                "side",
                "spacing",
                "startOffset",
                "textLength",
            ],
        ),
        ("title", SVG_NAMESPACE, &[]),
        (
            "tspan",
            SVG_NAMESPACE,
            &["dx", "dy", "lengthAdjust", "rotate", "textLength", "x", "y"],
        ),
    ];
    const ATTRIBUTES: &[&str] = &[
        "alignment-baseline",
        "baseline-shift",
        "clip-path",
        "clip-rule",
        "color",
        "color-interpolation",
        "cursor",
        "dir",
        "direction",
        "display",
        "displaystyle",
        "dominant-baseline",
        "fill",
        "fill-opacity",
        "fill-rule",
        "font-family",
        "font-size",
        "font-size-adjust",
        "font-stretch",
        "font-style",
        "font-variant",
        "font-weight",
        "lang",
        "letter-spacing",
        "marker-end",
        "marker-mid",
        "marker-start",
        "mathbackground",
        "mathcolor",
        "mathsize",
        "opacity",
        "paint-order",
        "pointer-events",
        "scriptlevel",
        "shape-rendering",
        "stop-color",
        "stop-opacity",
        "stroke",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-miterlimit",
        "stroke-opacity",
        "stroke-width",
        "text-anchor",
        "text-decoration",
        "text-overflow",
        "text-rendering",
        "title",
        "transform",
        "transform-origin",
        "unicode-bidi",
        "vector-effect",
        "visibility",
        "white-space",
        "word-spacing",
        "writing-mode",
    ];

    let create_attribute_vec = |attributes: &[&str]| -> Vec<SanitizerAttribute> {
        attributes
            .iter()
            .map(|&attribute| {
                SanitizerAttribute::SanitizerAttributeNamespace(SanitizerAttributeNamespace {
                    name: attribute.into(),
                    namespace: None,
                })
            })
            .collect()
    };

    let elements = ELEMENTS
        .iter()
        .map(|&(name, namespace, attributes)| {
            let attributes = create_attribute_vec(attributes);
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
                SanitizerElementNamespaceWithAttributes {
                    parent: SanitizerElementNamespace {
                        name: name.into(),
                        namespace: Some(namespace.into()),
                    },
                    attributes: Some(attributes),
                    removeAttributes: None,
                },
            )
        })
        .collect();

    let attributes = create_attribute_vec(ATTRIBUTES);

    SanitizerConfig {
        elements: Some(elements),
        removeElements: None,
        replaceWithChildrenElements: None,
        attributes: Some(attributes),
        removeAttributes: None,
        comments: Some(false),
        dataAttributes: Some(false),
    }
}
