/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/IntersectionObserver/#intersection-observer-entry

[Pref="dom_intersection_observer_enabled", Exposed=(Window)]
interface IntersectionObserverEntry {
  constructor(IntersectionObserverEntryInit intersectionObserverEntryInit);
  readonly attribute DOMHighResTimeStamp time;
  readonly attribute DOMRectReadOnly? rootBounds;
  readonly attribute DOMRectReadOnly boundingClientRect;
  readonly attribute DOMRectReadOnly intersectionRect;
  readonly attribute boolean isIntersecting;
  readonly attribute boolean isVisible;
  readonly attribute double intersectionRatio;
  readonly attribute Element target;
};

dictionary IntersectionObserverEntryInit {
  required DOMHighResTimeStamp time;
  required DOMRectInit rootBounds;
  required DOMRectInit boundingClientRect;
  required DOMRectInit intersectionRect;
  required boolean isIntersecting;
  required boolean isVisible;
  required double intersectionRatio;
  required Element target;
};
