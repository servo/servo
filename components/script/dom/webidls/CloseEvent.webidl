/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//https://html.spec.whatwg.org/multipage/#the-closeevent-interfaces
[Constructor(DOMString type, optional CloseEventInit eventInitDict), Exposed=(Window,Worker)]
interface CloseEvent : Event {
  readonly attribute boolean wasClean;
  readonly attribute unsigned short code;
  readonly attribute DOMString reason;
};

dictionary CloseEventInit : EventInit {
  boolean wasClean = false;
  unsigned short code = 0;
  DOMString reason = "";
};
