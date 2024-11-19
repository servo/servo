/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/uievents/#idl-compositionevent
 *
 */

// https://w3c.github.io/uievents/#idl-compositionevent
[Exposed=Window, Pref="dom.composition_event.enabled"]
interface CompositionEvent : UIEvent {
  [Throws] constructor(DOMString type, optional CompositionEventInit eventInitDict = {});
  readonly attribute DOMString data;
};

// https://w3c.github.io/uievents/#idl-compositioneventinit
dictionary CompositionEventInit : UIEventInit {
  DOMString data = "";
};

