/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-pagetransitionevent-interface
[Constructor(DOMString type, optional PageTransitionEventInit eventInitDict),
 Exposed=Window]
interface PageTransitionEvent : Event {
  readonly attribute boolean persisted;
};

dictionary PageTransitionEventInit : EventInit {
  boolean persisted = false;
};
