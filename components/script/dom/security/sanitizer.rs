/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{Namespace, ns};
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
    const ELEMENTS: &[(&str, &Namespace, &[&str])] = &[
        ("math", &ns!(mathml), &[]),
        ("merror", &ns!(mathml), &[]),
        ("mfrac", &ns!(mathml), &[]),
        ("mi", &ns!(mathml), &[]),
        ("mmultiscripts", &ns!(mathml), &[]),
        ("mn", &ns!(mathml), &[]),
        (
            "mo",
            &ns!(mathml),
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
        ("mover", &ns!(mathml), &["accent"]),
        (
            "mpadded",
            &ns!(mathml),
            &["depth", "height", "lspace", "voffset", "width"],
        ),
        ("mphantom", &ns!(mathml), &[]),
        ("mprescripts", &ns!(mathml), &[]),
        ("mroot", &ns!(mathml), &[]),
        ("mrow", &ns!(mathml), &[]),
        ("ms", &ns!(mathml), &[]),
        ("mspace", &ns!(mathml), &["depth", "height", "width"]),
        ("msqrt", &ns!(mathml), &[]),
        ("mstyle", &ns!(mathml), &[]),
        ("msub", &ns!(mathml), &[]),
        ("msubsup", &ns!(mathml), &[]),
        ("msup", &ns!(mathml), &[]),
        ("mtable", &ns!(mathml), &[]),
        ("mtd", &ns!(mathml), &["columnspan", "rowspan"]),
        ("mtext", &ns!(mathml), &[]),
        ("mtr", &ns!(mathml), &[]),
        ("munder", &ns!(mathml), &["accentunder"]),
        ("munderover", &ns!(mathml), &["accent", "accentunder"]),
        ("semantics", &ns!(mathml), &[]),
        ("a", &ns!(html), &["href", "hreflang", "type"]),
        ("abbr", &ns!(html), &[]),
        ("address", &ns!(html), &[]),
        ("article", &ns!(html), &[]),
        ("aside", &ns!(html), &[]),
        ("b", &ns!(html), &[]),
        ("bdi", &ns!(html), &[]),
        ("bdo", &ns!(html), &[]),
        ("blockquote", &ns!(html), &["cite"]),
        ("body", &ns!(html), &[]),
        ("br", &ns!(html), &[]),
        ("caption", &ns!(html), &[]),
        ("cite", &ns!(html), &[]),
        ("code", &ns!(html), &[]),
        ("col", &ns!(html), &["span"]),
        ("colgroup", &ns!(html), &["span"]),
        ("data", &ns!(html), &["value"]),
        ("dd", &ns!(html), &[]),
        ("del", &ns!(html), &["cite", "datetime"]),
        ("dfn", &ns!(html), &[]),
        ("div", &ns!(html), &[]),
        ("dl", &ns!(html), &[]),
        ("dt", &ns!(html), &[]),
        ("em", &ns!(html), &[]),
        ("figcaption", &ns!(html), &[]),
        ("figure", &ns!(html), &[]),
        ("footer", &ns!(html), &[]),
        ("h1", &ns!(html), &[]),
        ("h2", &ns!(html), &[]),
        ("h3", &ns!(html), &[]),
        ("h4", &ns!(html), &[]),
        ("h5", &ns!(html), &[]),
        ("h6", &ns!(html), &[]),
        ("head", &ns!(html), &[]),
        ("header", &ns!(html), &[]),
        ("hgroup", &ns!(html), &[]),
        ("hr", &ns!(html), &[]),
        ("html", &ns!(html), &[]),
        ("i", &ns!(html), &[]),
        ("ins", &ns!(html), &["cite", "datetime"]),
        ("kbd", &ns!(html), &[]),
        ("li", &ns!(html), &["value"]),
        ("main", &ns!(html), &[]),
        ("mark", &ns!(html), &[]),
        ("menu", &ns!(html), &[]),
        ("nav", &ns!(html), &[]),
        ("ol", &ns!(html), &["reversed", "start", "type"]),
        ("p", &ns!(html), &[]),
        ("pre", &ns!(html), &[]),
        ("q", &ns!(html), &[]),
        ("rp", &ns!(html), &[]),
        ("rt", &ns!(html), &[]),
        ("ruby", &ns!(html), &[]),
        ("s", &ns!(html), &[]),
        ("samp", &ns!(html), &[]),
        ("search", &ns!(html), &[]),
        ("section", &ns!(html), &[]),
        ("small", &ns!(html), &[]),
        ("span", &ns!(html), &[]),
        ("strong", &ns!(html), &[]),
        ("sub", &ns!(html), &[]),
        ("sup", &ns!(html), &[]),
        ("table", &ns!(html), &[]),
        ("tbody", &ns!(html), &[]),
        ("td", &ns!(html), &["colspan", "headers", "rowspan"]),
        ("tfoot", &ns!(html), &[]),
        (
            "th",
            &ns!(html),
            &["abbr", "colspan", "headers", "rowspan", "scope"],
        ),
        ("thead", &ns!(html), &[]),
        ("time", &ns!(html), &["datetime"]),
        ("title", &ns!(html), &[]),
        ("tr", &ns!(html), &[]),
        ("u", &ns!(html), &[]),
        ("ul", &ns!(html), &[]),
        ("var", &ns!(html), &[]),
        ("wbr", &ns!(html), &[]),
        ("a", &ns!(svg), &["href", "hreflang", "type"]),
        ("circle", &ns!(svg), &["cx", "cy", "pathLength", "r"]),
        ("defs", &ns!(svg), &[]),
        ("desc", &ns!(svg), &[]),
        (
            "ellipse",
            &ns!(svg),
            &["cx", "cy", "pathLength", "rx", "ry"],
        ),
        ("foreignObject", &ns!(svg), &["height", "width", "x", "y"]),
        ("g", &ns!(svg), &[]),
        ("line", &ns!(svg), &["pathLength", "x1", "x2", "y1", "y2"]),
        (
            "marker",
            &ns!(svg),
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
        ("metadata", &ns!(svg), &[]),
        ("path", &ns!(svg), &["d", "pathLength"]),
        ("polygon", &ns!(svg), &["pathLength", "points"]),
        ("polyline", &ns!(svg), &["pathLength", "points"]),
        (
            "rect",
            &ns!(svg),
            &["height", "pathLength", "rx", "ry", "width", "x", "y"],
        ),
        (
            "svg",
            &ns!(svg),
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
            &ns!(svg),
            &["dx", "dy", "lengthAdjust", "rotate", "textLength", "x", "y"],
        ),
        (
            "textPath",
            &ns!(svg),
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
        ("title", &ns!(svg), &[]),
        (
            "tspan",
            &ns!(svg),
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
                        namespace: Some(namespace.to_string().into()),
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
