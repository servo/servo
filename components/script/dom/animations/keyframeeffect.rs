/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use script_bindings::codegen::GenericUnionTypes::UnrestrictedDoubleOrKeyframeEffectOptions;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use script_bindings::root::{Dom, DomRoot};

use crate::dom::animationeffect::AnimationEffect;
use crate::dom::bindings::codegen::Bindings::KeyframeEffectBinding::KeyframeEffectMethods;
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
}

impl KeyframeEffect {
    pub(crate) fn new_inherited(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            animationeffect: AnimationEffect::new_inherited(),
            target_element: Default::default(),
        }
    }

    fn new_with_proto_and_cx(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(Self::new_inherited(window)),
            window,
            proto,
            cx,
        )
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

    /// <https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-setkeyframes>
    fn SetKeyframes(&self, _cx: &mut JSContext, _keyframes: *mut JSObject) {
        // > This effect’s set of keyframes is replaced with the result of performing the procedure to
        // > process a keyframes argument. If that procedure throws an exception, this effect’s
        // > keyframes are not modified.
        // FIXME: Implement this.
    }
}
