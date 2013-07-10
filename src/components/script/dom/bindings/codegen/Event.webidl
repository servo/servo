/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface please see
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional EventInit eventInitDict)]
interface Event {
  readonly attribute DOMString type;
  readonly attribute EventTarget? target;
  readonly attribute EventTarget? currentTarget;

  const unsigned short NONE = 0;
  const unsigned short CAPTURING_PHASE = 1;
  const unsigned short AT_TARGET = 2;
  const unsigned short BUBBLING_PHASE = 3;
  readonly attribute unsigned short eventPhase;

  void stopPropagation();
  void stopImmediatePropagation();

  readonly attribute boolean bubbles;
  readonly attribute boolean cancelable;
  void preventDefault();
  readonly attribute boolean defaultPrevented;

  readonly attribute boolean isTrusted;
  readonly attribute DOMTimeStamp timeStamp;

  [Throws]
  void initEvent(DOMString type, boolean bubbles, boolean cancelable);
};

/*// Mozilla specific legacy stuff.
partial interface Event {
  const long MOUSEDOWN    = 0x00000001;
  const long MOUSEUP      = 0x00000002;
  const long MOUSEOVER    = 0x00000004;
  const long MOUSEOUT     = 0x00000008;
  const long MOUSEMOVE    = 0x00000010;
  const long MOUSEDRAG    = 0x00000020;
  const long CLICK        = 0x00000040;
  const long DBLCLICK     = 0x00000080;
  const long KEYDOWN      = 0x00000100;
  const long KEYUP        = 0x00000200;
  const long KEYPRESS     = 0x00000400;
  const long DRAGDROP     = 0x00000800;
  const long FOCUS        = 0x00001000;
  const long BLUR         = 0x00002000;
  const long SELECT       = 0x00004000;
  const long CHANGE       = 0x00008000;
  const long RESET        = 0x00010000;
  const long SUBMIT       = 0x00020000;
  const long SCROLL       = 0x00040000;
  const long LOAD         = 0x00080000;
  const long UNLOAD       = 0x00100000;
  const long XFER_DONE    = 0x00200000;
  const long ABORT        = 0x00400000;
  const long ERROR        = 0x00800000;
  const long LOCATE       = 0x01000000;
  const long MOVE         = 0x02000000;
  const long RESIZE       = 0x04000000;
  const long FORWARD      = 0x08000000;
  const long HELP         = 0x10000000;
  const long BACK         = 0x20000000;
  const long TEXT         = 0x40000000;

  const long ALT_MASK     = 0x00000001;
  const long CONTROL_MASK = 0x00000002;
  const long SHIFT_MASK   = 0x00000004;
  const long META_MASK    = 0x00000008;

  readonly attribute EventTarget? originalTarget;
  readonly attribute EventTarget? explicitOriginalTarget;
  [ChromeOnly] readonly attribute boolean multipleActionsPrevented;

  void preventBubble();
  void preventCapture();
  boolean getPreventDefault();
  };*/

dictionary EventInit {
  boolean bubbles = false;
  boolean cancelable = false;
};

