/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-promiserejectionevent-interface

[Exposed=(Window,Worker)]
interface PromiseRejectionEvent : Event {
  [Throws] constructor(DOMString type, PromiseRejectionEventInit eventInitDict);
  readonly attribute object promise;
  readonly attribute any reason;
};

dictionary PromiseRejectionEventInit : EventInit {
  required object promise;
  any reason;
};
