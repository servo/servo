/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/uievents/#idl-compositionevent
 *
 */

// https://w3c.github.io/uievents/#idl-compositionevent
[Pref="dom.compositionevent.enabled", Constructor(DOMString type, optional CompositionEventInit eventInitDict)]
interface CompositionEvent : UIEvent {
  readonly attribute DOMString data;
};

// https://w3c.github.io/uievents/#idl-compositioneventinit
dictionary CompositionEventInit : UIEventInit {
  DOMString data = "";
};

