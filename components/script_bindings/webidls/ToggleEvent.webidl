/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-toggleevent-interface
[Exposed=Window]
interface ToggleEvent : Event {
  [Throws] constructor(DOMString type, optional ToggleEventInit eventInitDict = {});
  readonly attribute DOMString oldState;
  readonly attribute DOMString newState;
  readonly attribute Element? source;
};

dictionary ToggleEventInit : EventInit {
  DOMString oldState = "";
  DOMString newState = "";
  Element? source = null;
};
