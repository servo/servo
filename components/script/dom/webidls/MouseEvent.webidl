/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-MouseEvent
[Constructor(DOMString typeArg, optional MouseEventInit mouseEventInitDict)]
interface MouseEvent : UIEvent {
    readonly    attribute long           screenX;
    readonly    attribute long           screenY;
    readonly    attribute long           clientX;
    readonly    attribute long           clientY;
    readonly    attribute boolean        ctrlKey;
    readonly    attribute boolean        shiftKey;
    readonly    attribute boolean        altKey;
    readonly    attribute boolean        metaKey;
    readonly    attribute short          button;
    readonly    attribute EventTarget?   relatedTarget;
    // Introduced in DOM Level 3
    //readonly    attribute unsigned short buttons;
    //boolean getModifierState (DOMString keyArg);

    [Throws]
    readonly    attribute long           which;
};

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-MouseEventInit
dictionary MouseEventInit : SharedKeyboardAndMouseEventInit {
    long           screenX = 0;
    long           screenY = 0;
    long           clientX = 0;
    long           clientY = 0;
    short          button = 0;
    //unsigned short buttons = 0;
    EventTarget?   relatedTarget = null;
};

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-MouseEvent-1
partial interface MouseEvent {
    // Deprecated in DOM Level 3
    void initMouseEvent (DOMString typeArg, boolean bubblesArg, boolean cancelableArg,
                         Window? viewArg, long detailArg,
                         long screenXArg, long screenYArg,
                         long clientXArg, long clientYArg,
                         boolean ctrlKeyArg, boolean altKeyArg,
                         boolean shiftKeyArg, boolean metaKeyArg,
                         short buttonArg, EventTarget? relatedTargetArg);
};
