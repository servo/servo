/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/web-animations-1/#the-animatable-interface-mixin

interface mixin Animatable {
    Animation           animate(object? keyframes ,
                                optional (unrestricted double or KeyframeAnimationOptions) options = {});
    // sequence<Animation> getAnimations(optional GetAnimationsOptions options = {});
};

dictionary KeyframeAnimationOptions : KeyframeEffectOptions {
    DOMString id = "";
    AnimationTimeline? timeline;
};

dictionary GetAnimationsOptions {
    boolean subtree = false;
    CSSOMString? pseudoElement = null;
};
