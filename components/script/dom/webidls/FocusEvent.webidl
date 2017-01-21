/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/uievents/#interface-FocusEvent
[Constructor(DOMString typeArg, optional FocusEventInit focusEventInitDict),
 Exposed=Window]
interface FocusEvent : UIEvent {
  readonly attribute EventTarget?   relatedTarget;
};

dictionary FocusEventInit : UIEventInit {
    EventTarget? relatedTarget = null;
};
