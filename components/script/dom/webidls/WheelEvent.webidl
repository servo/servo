/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/uievents/#interface-wheelevent
[Constructor(DOMString typeArg, optional WheelEventInit wheelEventInitDict),
 Exposed=Window]
interface WheelEvent : MouseEvent {
    const unsigned long DOM_DELTA_PIXEL = 0x00;
    const unsigned long DOM_DELTA_LINE = 0x01;
    const unsigned long DOM_DELTA_PAGE = 0x02;
    readonly    attribute double         deltaX;
    readonly    attribute double         deltaY;
    readonly    attribute double         deltaZ;
    readonly    attribute unsigned long  deltaMode;
};

// https://w3c.github.io/uievents/#idl-wheeleventinit
dictionary WheelEventInit : MouseEventInit {
    double deltaX = 0.0;
    double deltaY = 0.0;
    double deltaZ = 0.0;
    unsigned long deltaMode = 0;
};

// https://w3c.github.io/uievents/#idl-interface-WheelEvent-initializers
partial interface WheelEvent {
    // Deprecated in DOM Level 3
    void initWheelEvent (DOMString typeArg, boolean bubblesArg, boolean cancelableArg,
                         Window? viewArg, long detailArg,
                         double deltaX, double deltaY,
                         double deltaZ, unsigned long deltaMode);
};
