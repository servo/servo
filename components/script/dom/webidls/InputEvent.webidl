/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/uievents/#idl-inputevent
 *
 */

// https://w3c.github.io/uievents/#idl-inputevent
[Constructor(DOMString type, optional InputEventInit eventInitDict)]
interface InputEvent : UIEvent {
  readonly attribute DOMString? data;
  readonly attribute boolean isComposing;
};

// https://w3c.github.io/uievents/#idl-inputeventinit
dictionary InputEventInit : UIEventInit {
  DOMString? data = null;
  boolean isComposing = false;
};
