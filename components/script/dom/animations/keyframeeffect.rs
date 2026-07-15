/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::ops::ControlFlow;
use std::ptr::{self, NonNull};
use std::sync::LazyLock;

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::{
    ConversionResult, FromJSValConvertible, ToJSValConvertible, jsstr_to_string,
};
use js::gc::{HandleValue, RootedVec};
use js::jsapi::{HandleId, Heap, JS_GetPropertyById, JSITER_OWNONLY, JSObject, JSPROP_ENUMERATE};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers2::{GetPropertyKeys, JS_DefineProperty, JS_IdToValue, JS_NewObject};
use js::rust::{
    ForOfIterationFailure, HandleObject, IdVector, IntoHandle, IntoMutableHandle, for_of,
};
use rustc_hash::FxHashMap;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::KeyframeEffectBinding::{
    BaseKeyframe, CompositeOperationOrAuto,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::codegen::GenericUnionTypes::UnrestrictedDoubleOrKeyframeEffectOptions;
use script_bindings::conversions::StringificationBehavior;
use script_bindings::error::{Error, Fallible};
use script_bindings::num::Finite;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use style::parser::ParserContext;
use style::properties::generated::PropertyDeclaration;
use style::properties::{
    Importance, LonghandId, NonCustomPropertyId, PropertyDeclarationBlock, PropertyId,
    SourcePropertyDeclaration,
};
use style::stylesheets::CssRuleType;
use style_traits::{CssWriter, ParsingMode, ToCss};

use crate::css::parser_context_for_document;
use crate::dom::Document;
use crate::dom::animationeffect::AnimationEffect;
use crate::dom::bindings::codegen::Bindings::KeyframeEffectBinding::{
    BaseComputedKeyframe, KeyframeEffectMethods,
};
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::element::Element;
use crate::dom::window::Window;

/// <https://drafts.csswg.org/web-animations-1/#keyframeeffect>
#[dom_struct]
pub(crate) struct KeyframeEffect {
    animationeffect: AnimationEffect,

    /// The window that this keyframe was constructed in
    window: Dom<Window>,

    /// <https://drafts.csswg.org/web-animations-1/#effect-target-target-element>
    // FIXME: Store a target pseudo-selector
    // to fully match the concept of the effect target
    //
    // https://drafts.csswg.org/web-animations-1/#effect-target-target-pseudo-selector
    // https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target.
    target_element: MutNullableDom<Element>,

    /// <https://drafts.csswg.org/web-animations-1/#keyframe>
    keyframes: DomRefCell<Vec<Keyframe>>,
}

impl KeyframeEffect {
    pub(crate) fn new_inherited(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            animationeffect: AnimationEffect::new_inherited(),
            target_element: Default::default(),
            keyframes: Default::default(),
        }
    }

    fn new_with_proto_and_cx(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(cx, Box::new(Self::new_inherited(window)), window, proto)
    }

    pub(crate) fn new(cx: &mut JSContext, window: &Window) -> DomRoot<Self> {
        Self::new_with_proto_and_cx(cx, window, None)
    }
}

impl KeyframeEffectMethods<crate::DomTypeHolder> for KeyframeEffect {
    /// <https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-keyframeeffect>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        _: Option<HandleObject>,
        target: Option<&Element>,
        keyframes: *mut JSObject,
        _options: UnrestrictedDoubleOrKeyframeEffectOptions,
    ) -> DomRoot<KeyframeEffect> {
        // Step 1. Create a new KeyframeEffect object, effect.
        let effect = KeyframeEffect::new(cx, window);

        // Step 2. Set the target element of effect to target.
        effect.target_element.set(target);

        // TODO: Step 3. Set the target pseudo-selector to the result corresponding to
        // the first matching condition below:

        // TODO: Step 4. Let timing input be the result corresponding to the first matching
        // condition below:

        // Step 5. Call the procedure to update the timing properties of an animation effect of
        // effect from timing input.
        // If that procedure causes an exception to be thrown, propagate the exception and abort this procedure.

        // TODO: Step 6. If options is a KeyframeEffectOptions object, assign the composite property of effect
        // to the corresponding value from options.
        //
        // When assigning this property, the error-handling defined for the corresponding setter on the
        //  KeyframeEffect interface is applied. If the setter requires an exception to be thrown for the value
        //  specified by options, this procedure must throw the same exception and abort all further steps.

        // Step 7. Initialize the set of keyframes by performing the procedure defined for setKeyframes()
        // passing keyframes as the input.
        effect.SetKeyframes(cx, keyframes);

        effect
    }

    /// <https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-getkeyframes>
    #[expect(unsafe_code)]
    fn GetKeyframes(
        &self,
        cx: &mut JSContext,
        result: &mut RootedVec<'_, Box<Heap<*mut JSObject>>>,
    ) -> Fallible<()> {
        let mut layout = self.window.layout_mut();
        let stylist = layout.stylist_mut();

        // Step 1. Let result be an empty sequence of objects.
        debug_assert!(result.is_empty());

        // Step 2. Let keyframes be one of the following:
        // If this keyframe effect is associated with a CSSAnimation, and its keyframes have not been replaced
        // by a successful call to setKeyframes(),
        // the computed keyframes for this keyframe effect.
        // Otherwise,
        // the result of applying the procedure compute missing keyframe offsets to the keyframes for this keyframe effect.
        // TODO: We don't compute missing keyframe offsets yet. But that will likely happen in stylo, not here.
        let keyframes = self.keyframes.borrow();

        // Step 3. For each keyframe in keyframes perform the following steps:
        for keyframe in keyframes.iter() {
            // Step 3.1 Initialize a dictionary object, output keyframe, using the following definition:
            // TODO Step 3.2 Set the offset, computedOffset, easing, and composite members of output keyframe
            // to the respective keyframe offset, computed keyframe offset, keyframe-specific easing function,
            // and keyframe-specific composite operation values of keyframe.
            let base_keyframe = BaseComputedKeyframe {
                composite: keyframe.composite,
                offset: keyframe.offset,
                // FIXME: We don't post-process the offset of keyframes to find suitable offset values for null
                // keyframe offsets yet, so we just use the offset as-is.
                computedOffset: keyframe.offset,
                easing: keyframe.easing_function.clone(),
            };
            rooted!(&in(cx) let mut output_keyframe = unsafe { JS_NewObject(cx, ptr::null()) });
            base_keyframe.to_jsobject(cx, output_keyframe.handle_mut());

            // Step 3.3 For each animation property-value pair declaration in keyframe, perform the following steps:
            for property_value_pair in &keyframe.declarations {
                debug_assert!(property_value_pair.property_id.is_animatable());

                // Step 3.3.1 Let property name be the result of applying the animation property name to IDL attribute
                // name algorithm to the property name of declaration.
                let mut property_name = String::new();
                let mut writer = CssWriter::new(&mut property_name);
                if property_value_pair.property_id.to_css(&mut writer).is_err() {
                    continue;
                }
                let property_name = animation_property_name_to_idl_attribute_name(&property_name);

                // Step 3.3.2 Let IDL value be the result of serializing the property value of declaration
                // by passing declaration to the algorithm to serialize a CSS value [CSSOM].
                let mut value_string = String::new();
                if property_value_pair
                    .block
                    .single_value_to_css(
                        &property_value_pair.property_id,
                        &mut value_string,
                        None,
                        stylist,
                    )
                    .is_err()
                {
                    continue;
                }

                // Step 3.3.3 Let value be the result of converting IDL value to an ECMAScript String value.
                rooted!(&in(cx) let mut value = UndefinedValue());
                value_string.safe_to_jsval(cx, value.handle_mut());

                // Step 3.3.4 Call the [[DefineOwnProperty]] internal method on output keyframe with property
                // name property name, Property Descriptor { [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]:
                // true, [[Value]]: value } and Boolean flag false.
                let Ok(property_name) = CString::new(property_name) else {
                    continue;
                };

                let success = unsafe {
                    JS_DefineProperty(
                        cx,
                        output_keyframe.handle(),
                        property_name.as_ptr(),
                        value.handle(),
                        JSPROP_ENUMERATE as u32,
                    )
                };
                if !success {
                    if cfg!(debug_assertions) {
                        unreachable!("Setting a property on output_keyframe should never fail");
                    }
                    return Err(Error::Operation(None));
                }
            }

            // Step 3.4 Append output keyframe to result.
            result.push(Heap::boxed(output_keyframe.get()))
        }

        // Step 4. Return result.
        Ok(())
    }

    /// <https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-setkeyframes>
    fn SetKeyframes(&self, cx: &mut JSContext, keyframes: *mut JSObject) {
        // > This effect’s set of keyframes is replaced with the result of performing the procedure to
        // > process a keyframes argument. If that procedure throws an exception, this effect’s
        // > keyframes are not modified.
        let document = self.window.Document();
        let Ok(keyframes) = process_a_keyframes_argument(cx, &document, keyframes) else {
            return;
        };
        *self.keyframes.safe_borrow_mut(cx.no_gc()) = keyframes;
    }
}

/// <https://drafts.csswg.org/web-animations-1/#process-a-keyframes-argument>
#[expect(unsafe_code)]
fn process_a_keyframes_argument(
    cx: &mut JSContext,
    document: &Document,
    keyframes: *mut JSObject,
) -> Fallible<Vec<Keyframe>> {
    // Step 1. If object is null, return an empty sequence of keyframes.
    if keyframes.is_null() {
        return Ok(Vec::new());
    }

    // Step 2. Let processed keyframes be an empty sequence of keyframes.

    // Step 3. Let method be the result of GetMethod(object, @@iterator).
    // Step 4. Check the completion record of method.
    // Step 5. Perform the steps corresponding to the first matching condition below:
    rooted!(&in(cx) let iterable = ObjectValue(keyframes));
    let mut keyframes = Vec::new();
    let result = for_of(
        unsafe { cx.raw_cx() },
        iterable.handle(),
        |iterator_element| {
            // Step 5.3.6 If Type(nextItem) is not Undefined, Null or Object, then throw a TypeError
            // and abort these steps.
            //
            // Note: nextItem is later passed to "process a keyframe like object" which cannot handle undefined
            // or null values. This seems to be a bug in the specification which is tracked by
            // https://github.com/w3c/csswg-drafts/issues/14113
            if !iterator_element.is_object() {
                return Err(ForOfIterationFailure::Other(Error::Type(
                    c"Keyframe must be an object".to_owned(),
                )));
            }

            // Step 5.3.7 Append to processed keyframes the result of running the procedure to process a
            // keyframe-like object passing nextItem as the keyframe input with the allow lists flag set to false.
            keyframes.push(keyframe_from_value(cx, document, iterator_element)?);

            Ok(ControlFlow::Continue(()))
        },
    );
    match result {
        Ok(()) => Ok(keyframes),
        Err(ForOfIterationFailure::ValueIsNotIterable) => {
            // TODO: Step 5, Otherwise:
            Err(Error::Operation(None))
        },
        Err(ForOfIterationFailure::JSFailed) => Err(Error::JSFailed),
        Err(ForOfIterationFailure::Other(error)) => Err(error),
    }
}

/// <https://drafts.csswg.org/web-animations-1/#keyframe>
#[derive(JSTraceable, MallocSizeOf)]
struct Keyframe {
    offset: Option<Finite<f64>>,
    easing_function: DOMString,
    composite: CompositeOperationOrAuto,
    declarations: Vec<KeyframePropertyDeclaration>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct KeyframePropertyDeclaration {
    #[no_trace]
    property_id: PropertyId,
    /// The block is known to only contain declarations for `property_id`. There might
    /// be more than one value if `property_id` is a shorthand.
    #[no_trace]
    block: PropertyDeclarationBlock,
}

/// Step 5 (for iterable keyframes) of <https://drafts.csswg.org/web-animations-1/#process-a-keyframes-argument>.
fn keyframe_from_value(
    cx: &mut JSContext,
    document: &Document,
    value: HandleValue<'_>,
) -> Fallible<Keyframe> {
    // Step 3.4 Let nextItem be IteratorValue(next).
    // NOTE: This is "current_value"
    // Step 3.5 Check the completion record of nextItem.

    // Step 3.6 If Type(nextItem) is not Undefined, Null or Object,
    // then throw a TypeError and abort these steps.
    if !value.is_null_or_undefined() && !value.is_object() {
        return Err(Error::Type(c"Invalid keyframe value".to_owned()));
    }

    // Step 3.7 Append to processed keyframes the result of running the procedure to process
    // a keyframe-like object passing nextItem as the keyframe input with the allow lists
    // flag set to false.
    process_a_keyframe_like_object(cx, document, value)
}

/// <https://drafts.csswg.org/web-animations-1/#process-a-keyframe-like-object>
fn process_a_keyframe_like_object(
    cx: &mut JSContext,
    document: &Document,
    value: HandleValue,
) -> Fallible<Keyframe> {
    // Step 1. Run the procedure to convert an ECMAScript value to a dictionary type [WEBIDL] with keyframe input
    // as the ECMAScript value, and the dictionary type depending on the value of the allow lists flag as follows:
    // If allow lists is true,
    // Use the following dictionary type:
    //
    // dictionary BasePropertyIndexedKeyframe {
    //   (double? or sequence<double?>)                         offset = [];
    //   (DOMString or sequence<DOMString>)                     easing = [];
    //   (CompositeOperationOrAuto or sequence<CompositeOperationOrAuto>) composite = [];
    // };
    //
    // Otherwise,
    //     Use the following dictionary type:
    //
    //     dictionary BaseKeyframe {
    //       double?                  offset = null;
    //       DOMString                easing = "linear";
    //       CompositeOperationOrAuto composite = "auto";
    //     };
    //
    // Store the result of this procedure as keyframe output.
    //
    // Note: 'allow lists' is currently never true.
    // Use the following dictionary type:
    let Ok(keyframe_output) = BaseKeyframe::safe_from_jsval(cx, value, ()) else {
        return Err(Error::JSFailed);
    };
    let ConversionResult::Success(keyframe_output) = keyframe_output else {
        return Err(Error::Operation(None));
    };

    // From Step 2 onwards our implementation diverges from the specification. The spec
    // wants us to build a list of animatable CSS properties and a list of properties on
    // the object, then compute the union between the two.
    //
    // Instead, we iterate over all properties on the object and then check if they correspond
    // to an animatable property.
    let urlextradata = document.url().into_url().into();
    let parser_context = parser_context_for_document(
        document,
        CssRuleType::Style,
        ParsingMode::DEFAULT,
        &urlextradata,
    );
    rooted!(&in(cx) let object = value.to_object());

    // Steps 2 - 6 are in get_property_declarations
    let declarations = get_property_declarations(cx, object.handle(), &parser_context)?;

    // Step 7. Return keyframe output.
    Ok(Keyframe {
        offset: keyframe_output.offset,
        easing_function: keyframe_output.easing,
        composite: keyframe_output.composite,
        declarations,
    })
}

/// Implements Step 2-6 of  <https://drafts.csswg.org/web-animations-1/#process-a-keyframe-like-object>.
#[expect(unsafe_code)]
fn get_property_declarations(
    cx: &mut JSContext,
    object: HandleObject,
    parser_context: &ParserContext<'_>,
) -> Fallible<Vec<KeyframePropertyDeclaration>> {
    // The spec tells us to iterate over all animatable properties and see if they're defined
    // on the object. Instead we can iterate over the own properties of the object and see
    // if they're animated properties, that's easier.
    let mut ids = unsafe { IdVector::new(cx.raw_cx()) };
    if !unsafe { GetPropertyKeys(cx, object, JSITER_OWNONLY, ids.handle_mut()) } {
        return Ok(Vec::new());
    }

    let mut declarations = Vec::with_capacity(ids.len());
    for id in ids.iter() {
        rooted!(&in(cx) let id = *id);

        // See if the id for the current property on the object represents a animatable CSS property.
        if !id.is_string() {
            continue;
        }
        rooted!(&in(cx) let mut key_value = UndefinedValue());
        let raw_id: HandleId = id.handle().into();
        if !unsafe { JS_IdToValue(cx, *raw_id.ptr, key_value.handle_mut()) } {
            continue;
        }
        rooted!(&in(cx) let js_string = key_value.to_string());
        let Some(js_string) = NonNull::new(js_string.get()) else {
            continue;
        };
        let property_name = unsafe { jsstr_to_string(cx, js_string) };

        let Some(property_id) = lookup_css_property_by_idl_attribute_name(&property_name) else {
            continue;
        };
        debug_assert!(property_id.is_animatable());

        // Step 6.1 Let raw value be the result of calling the [[Get]] internal method on keyframe input,
        // with property name as the property key and keyframe input as the receiver.
        // Step 6.2 Check the completion record of raw value.
        rooted!(&in(cx) let mut property_value = UndefinedValue());
        if !unsafe {
            JS_GetPropertyById(
                cx.raw_cx(),
                object.into_handle(),
                id.handle().into_handle(),
                property_value.handle_mut().into_handle_mut(),
            )
        } {
            continue;
        }

        // Step 6.3 Convert raw value to a DOMString or to a sequence of DOMStrings property values as follows:
        // If allow lists is true, [..] (Note: We don't implement "allow lists")
        // Otherwise,
        // Let property values be the result of converting raw value to a DOMString using the procedure
        // for converting an ECMAScript value to a DOMString [WEBIDL].
        let property_value = match DOMString::safe_from_jsval(
            cx,
            property_value.handle(),
            StringificationBehavior::Default,
        ) {
            Ok(ConversionResult::Success(property_value)) => property_value,
            Ok(ConversionResult::Failure(error_message)) => {
                return Err(Error::Operation(
                    error_message
                        .to_str()
                        .ok()
                        .map(|message| message.to_owned()),
                ));
            },
            Err(_) => return Err(Error::JSFailed),
        };

        // Step 6.4 Calculate the normalized property name as the result of applying the IDL attribute name
        // to animation property name algorithm to property name.
        // Note: Due to the way our implementation differs from the spec (refer to the comment at
        // the top of this function), we already have the normalized property name.

        // Parse the property value as a value for the given animatable CSS property.
        // The specification continues to treat the value as a plain string, but there's not much point.
        let Some(declaration) =
            parse_single_property_declaration(property_id, &property_value.str(), parser_context)
        else {
            continue;
        };

        // Step 6.5 Add a property to keyframe output with normalized property name as the property name,
        // and property values as the property value.
        declarations.push(declaration);
    }

    Ok(declarations)
}

/// Parses `input` as a value for `property`, returning `None` on failure.
fn parse_single_property_declaration(
    property: NonCustomPropertyId,
    input: &str,
    parser_context: &ParserContext<'_>,
) -> Option<KeyframePropertyDeclaration> {
    let mut declaration = SourcePropertyDeclaration::default();
    let mut input = ParserInput::new(input);
    let mut parser = Parser::new(&mut input);

    // TODO: Consider reporting parse errors somewhere useful, like the devtools console.
    parser
        .parse_entirely(|parser| {
            PropertyDeclaration::parse_into(
                &mut declaration,
                PropertyId::NonCustom(property),
                parser_context,
                parser,
            )
        })
        .ok()?;

    let mut block = PropertyDeclarationBlock::new();
    block.extend(declaration.drain(), Importance::Normal);

    Some(KeyframePropertyDeclaration {
        property_id: PropertyId::NonCustom(property),
        block,
    })
}

/// This is the inverse of <https://drafts.csswg.org/web-animations-1/#animation-property-name-to-idl-attribute-name>.
fn lookup_css_property_by_idl_attribute_name(attribute_name: &str) -> Option<NonCustomPropertyId> {
    // TODO: Step 1. If property follows the <custom-property-name> production, return property.

    // Step 2. If property refers to the CSS float property, return the string "cssFloat".
    if attribute_name == "cssFloat" {
        return Some(LonghandId::Float.into());
    }

    // Step 3. If property refers to the CSS offset property, return the string "cssOffset".
    // NOTE: "offset" is not supported yet.

    // Step 4. Otherwise, return the result of applying the CSS property to IDL attribute algorithm
    // [CSSOM] to property.
    static IDL_ATTRIBUTE_TO_ANIMATED_PROPERTY_LOOKUP_TABLE: LazyLock<
        FxHashMap<String, NonCustomPropertyId>,
    > = LazyLock::new(|| {
        log::debug!("Initializing map from IDL attribute names to CSS properties");

        NonCustomPropertyId::iter()
            .filter(|non_custom_property| non_custom_property.is_animatable())
            .filter(|non_custom_property| {
                non_custom_property
                    .to_property_id()
                    .enabled_for_all_content()
            })
            .map(|non_custom_property| {
                let idl_attribute_name =
                    animation_property_name_to_idl_attribute_name(non_custom_property.name());
                (idl_attribute_name, non_custom_property)
            })
            .collect()
    });
    IDL_ATTRIBUTE_TO_ANIMATED_PROPERTY_LOOKUP_TABLE
        .get(attribute_name)
        .copied()
}

/// <https://drafts.csswg.org/web-animations-1/#animation-property-name-to-idl-attribute-name>
///
/// # Panics
/// Panics when the property name is empty or consists only of `-` characters.
/// In general it is assumed that `property_name` is a CSS property.
fn animation_property_name_to_idl_attribute_name(property_name: &str) -> String {
    let mut idl_attribute_name = String::with_capacity(property_name.len());

    let mut chunks = property_name.split('-');
    let Some(first_chunk) = chunks.next() else {
        unreachable!("CSS property name should not consist only of dashes");
    };
    idl_attribute_name.push_str(first_chunk);
    for chunk in chunks {
        let mut characters = chunk.chars();
        let Some(to_capitalize) = characters.next() else {
            continue;
        };
        idl_attribute_name.push(to_capitalize.to_ascii_uppercase());
        idl_attribute_name.push_str(characters.as_str());
    }

    idl_attribute_name
}
