/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/web-animations-1/#animationeffect
[Exposed=Window, Pref="dom_web_animations_enabled"]
interface AnimationEffect {
//     EffectTiming         getTiming();
//     ComputedEffectTiming getComputedTiming();
//     undefined            updateTiming(optional OptionalEffectTiming timing = {});
};

// https://drafts.csswg.org/web-animations-1/#the-effecttiming-dictionaries
dictionary EffectTiming {
  double                             delay = 0;
  double                             endDelay = 0;
  FillMode                           fill = "auto";
  double                             iterationStart = 0.0;
  unrestricted double                iterations = 1.0;
  (unrestricted double or DOMString) duration = "auto";
  PlaybackDirection                  direction = "normal";
  DOMString                          easing = "linear";
};

dictionary OptionalEffectTiming {
  double                             delay;
  double                             endDelay;
  FillMode                           fill;
  double                             iterationStart;
  unrestricted double                iterations;
  (unrestricted double or DOMString) duration;
  PlaybackDirection                  direction;
  DOMString                          easing;
};

// https://drafts.csswg.org/web-animations-1/#the-fillmode-enumeration
enum FillMode { "none", "forwards", "backwards", "both", "auto" };

// https://drafts.csswg.org/web-animations-1/#the-playbackdirection-enumeration
enum PlaybackDirection { "normal", "reverse", "alternate", "alternate-reverse" };
