/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
[Exposed=Window]
interface PopStateEvent : Event {
  [Throws] constructor(DOMString type, optional PopStateEventInit eventInitDict = {});
  readonly attribute any state;
};

dictionary PopStateEventInit : EventInit {
  any state = null;
};
