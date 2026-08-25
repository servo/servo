/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/web-animations-1/#keyframeeffect
[Exposed=Window, Pref="dom_web_animations_enabled"]
interface KeyframeEffect : AnimationEffect {
  constructor(Element?       target,
              object?        keyframes,
              optional (unrestricted double or KeyframeEffectOptions) options = {});
//   constructor(KeyframeEffect source);
//   attribute Element?           target;
//   attribute CSSOMString?       pseudoElement;
//   attribute CompositeOperation composite;
   [Throws] sequence<object> getKeyframes();
   undefined        setKeyframes(object? keyframes);
};

// https://drafts.csswg.org/web-animations-1/#dictdef-keyframeeffectoptions
dictionary KeyframeEffectOptions : EffectTiming {
    CompositeOperation composite = "replace";
    CSSOMString?       pseudoElement = null;
};

// https://drafts.csswg.org/web-animations-1/#the-compositeoperation-enumeration
enum CompositeOperation { "replace", "add", "accumulate" };
enum CompositeOperationOrAuto { "replace", "add", "accumulate", "auto" };

// Necessary for https://drafts.csswg.org/web-animations-1/#process-a-keyframe-like-object
dictionary BasePropertyIndexedKeyframe {
//   (double? or sequence<double?>)                         offset = [];
//   (DOMString or sequence<DOMString>)                     easing = [];
//   (CompositeOperationOrAuto or sequence<CompositeOperationOrAuto>) composite = [];
};
dictionary BaseKeyframe {
   double?                  offset = null;
   DOMString                easing = "linear";
   CompositeOperationOrAuto composite = "auto";
};

// Part of https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-getkeyframes
dictionary BaseComputedKeyframe {
     double?                  offset = null;
     double                   computedOffset;
     DOMString                easing = "linear";
     CompositeOperationOrAuto composite = "auto";
};
