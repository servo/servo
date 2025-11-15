/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-commandevent-interface
[Exposed=Window, Pref="dom_command_invokers_enabled"]
interface CommandEvent : Event {
  [Throws] constructor(DOMString type, optional CommandEventInit eventInitDict = {});
  readonly attribute Element? source;
  readonly attribute DOMString command;
};

dictionary CommandEventInit : EventInit {
  Element? source = null;
  DOMString command = "";
};
