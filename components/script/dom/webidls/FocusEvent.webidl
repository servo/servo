/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-FocusEvent
[Constructor(DOMString typeArg, optional FocusEventInit focusEventInitDict)]
interface FocusEvent : UIEvent {
  readonly attribute EventTarget?   relatedTarget;
};

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-FocusEventInit
dictionary FocusEventInit : UIEventInit {
    EventTarget? relatedTarget = null;
};

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-FocusEvent-1
partial interface FocusEvent {
    // Deprecated in DOM Level 3
    void initFocusEvent (DOMString typeArg, boolean bubblesArg, boolean cancelableArg,
                         Window? viewArg, long detailArg,
                         EventTarget? relatedTargetArg);
};
