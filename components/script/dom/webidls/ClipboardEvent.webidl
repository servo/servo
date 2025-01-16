/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/clipboard-apis/

[Exposed=Window, Pref="dom_clipboardevent_enabled"]
interface ClipboardEvent : Event {
  constructor (DOMString type, optional ClipboardEventInit eventInitDict = {});
  readonly attribute DataTransfer? clipboardData;
};

dictionary ClipboardEventInit : EventInit {
  DataTransfer? clipboardData = null;
};
