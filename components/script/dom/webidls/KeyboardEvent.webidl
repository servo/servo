/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#interface-KeyboardEvent
 *
 */

[Constructor(DOMString typeArg, optional KeyboardEventInit keyboardEventInitDict)]
interface KeyboardEvent : UIEvent {
    // KeyLocationCode
    const unsigned long DOM_KEY_LOCATION_STANDARD = 0x00;
    const unsigned long DOM_KEY_LOCATION_LEFT = 0x01;
    const unsigned long DOM_KEY_LOCATION_RIGHT = 0x02;
    const unsigned long DOM_KEY_LOCATION_NUMPAD = 0x03;
  //readonly    attribute DOMString     key;
  //readonly    attribute DOMString     code;
  //readonly    attribute unsigned long location;
  //readonly    attribute boolean       ctrlKey;
  //readonly    attribute boolean       shiftKey;
  //readonly    attribute boolean       altKey;
  //readonly    attribute boolean       metaKey;
  //readonly    attribute boolean       repeat;
  //readonly    attribute boolean       isComposing;
  //boolean getModifierState (DOMString keyArg);
};

dictionary KeyboardEventInit : SharedKeyboardAndMouseEventInit {
    DOMString     key = "";
    DOMString     code = "";
    unsigned long location = 0;
    boolean       repeat = false;
    boolean       isComposing = false;
};
