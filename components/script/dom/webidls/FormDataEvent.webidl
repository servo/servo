/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-formdataevent-interface
[Exposed=Window,
 Constructor(DOMString type, optional FormDataEventInit eventInitDict)]
interface FormDataEvent : Event {
  readonly attribute FormData formData;
};

dictionary FormDataEventInit : EventInit {
  /*required*/ FormData formData;
};
