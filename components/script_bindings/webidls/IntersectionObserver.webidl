/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/IntersectionObserver/#intersection-observer-interface

callback IntersectionObserverCallback =
  undefined (sequence<IntersectionObserverEntry> entries, IntersectionObserver observer);

dictionary IntersectionObserverInit {
  (Element or Document)?  root = null;
  DOMString rootMargin;
  DOMString scrollMargin;
  (double or sequence<double>) threshold;
  long delay;
  boolean trackVisibility = false;
};

[Pref="dom_intersection_observer_enabled", Exposed=(Window)]
interface IntersectionObserver {
  [Throws] constructor(IntersectionObserverCallback callback, optional IntersectionObserverInit options = {});
  readonly attribute (Element or Document)? root;
  readonly attribute DOMString rootMargin;
  readonly attribute DOMString scrollMargin;
  readonly attribute /* FrozenArray<double> */ any thresholds;
  readonly attribute long delay;
  readonly attribute boolean trackVisibility;
  undefined observe(Element target);
  undefined unobserve(Element target);
  undefined disconnect();
  sequence<IntersectionObserverEntry> takeRecords();
};
