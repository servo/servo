/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use script_bindings::root::DomRoot;

use crate::dom::animationeffect::AnimationEffect;
use crate::dom::bindings::codegen::Bindings::AnimationBinding::AnimationMethods;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::AnimationTimeline;
use crate::dom::window::Window;

/// <https://drafts.csswg.org/web-animations-1/#animation>
#[dom_struct]
pub(crate) struct Animation {
    event_target: EventTarget,

    /// <https://drafts.csswg.org/web-animations-1/#timeline>
    timeline: MutNullableDom<AnimationTimeline>,

    /// <https://drafts.csswg.org/web-animations-1/#animation-associated-effect>
    associated_effect: MutNullableDom<AnimationEffect>,
}

impl Animation {
    pub(crate) fn new_inherited() -> Self {
        Self {
            event_target: EventTarget::new_inherited(),
            timeline: Default::default(),
            associated_effect: Default::default(),
        }
    }

    fn new_with_proto_and_cx(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_cx(Box::new(Self::new_inherited()), global, proto, cx)
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<Self> {
        Self::new_with_proto_and_cx(cx, global, None)
    }

    /// <https://drafts.csswg.org/web-animations-1/#animation-set-the-timeline-of-an-animation>
    fn set_the_timeline(&self, timeline: &AnimationTimeline) {
        // FIXME: Implement this fully
        self.timeline.set(Some(timeline));
    }

    /// <https://drafts.csswg.org/web-animations-1/#animation-set-the-associated-effect-of-an-animation>
    fn set_the_associated_effect(&self, effect: Option<&AnimationEffect>) {
        // FIXME: Implement this fully
        self.associated_effect.set(effect);
    }
}

impl AnimationMethods<crate::DomTypeHolder> for Animation {
    /// <https://drafts.csswg.org/web-animations-1/#dom-animation-animation>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        _object: Option<HandleObject>,
        effect: Option<&AnimationEffect>,
    ) -> DomRoot<Self> {
        let document = window.Document();

        // Step 1. Let animation be a new Animation object.
        let animation = Animation::new(cx, window.upcast());

        // Step 2. Run the procedure to set the timeline of an animation on animation passing timeline
        // as the new timeline; or, if the timeline argument is missing, passing the default document
        // timeline of the Document associated with the Window that is the current global object.
        // TODO: We don't suppor the timeline argument yet.
        animation.set_the_timeline(document.Timeline().upcast());

        // Step 3. Run the procedure to set the associated effect of an animation on animation passing
        // source as the new effect.
        animation.set_the_associated_effect(effect);

        animation
    }
}
