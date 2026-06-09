/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/web-animations-1/#animation

[Exposed=Window]
interface Animation : EventTarget {
  constructor(optional AnimationEffect? effect = null/*,
              optional AnimationTimeline? timeline */);
  //          attribute DOMString                id;
  //          attribute AnimationEffect?         effect;
  //          attribute AnimationTimeline?       timeline;
  //          attribute double?                  startTime;
  //          attribute double?                  currentTime;
  //          attribute double                   playbackRate;
  // readonly attribute AnimationPlayState       playState;
  // readonly attribute AnimationReplaceState    replaceState;
  // readonly attribute boolean                  pending;
  // readonly attribute Promise<Animation>       ready;
  // readonly attribute Promise<Animation>       finished;
  //          attribute EventHandler             onfinish;
  //          attribute EventHandler             oncancel;
  //          attribute EventHandler             onremove;
  // undefined cancel();
  // undefined finish();
  // undefined play();
  // undefined pause();
  // undefined updatePlaybackRate(double playbackRate);
  // undefined reverse();
  // undefined persist();
  // [CEReactions]
  // undefined commitStyles();
};
