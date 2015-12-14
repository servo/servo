/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * For more information on this interface please see
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional EventInit eventInitDict)]
interface Event {
  [Pure]
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

  [Pure]
  readonly attribute boolean bubbles;
  [Pure]
  readonly attribute boolean cancelable;
  void preventDefault();
  [Pure]
  readonly attribute boolean defaultPrevented;

  [Unforgeable]
  readonly attribute boolean isTrusted;
  [Constant]
  readonly attribute DOMTimeStamp timeStamp;

  void initEvent(DOMString type, boolean bubbles, boolean cancelable);
};

dictionary EventInit {
  boolean bubbles = false;
  boolean cancelable = false;
};
