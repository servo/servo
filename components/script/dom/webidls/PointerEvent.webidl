/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.w3.org/TR/pointerevents/#pointer-events-and-interfaces
[Constructor(DOMString type, optional PointerEventInit eventInitDict)]
interface PointerEvent : MouseEvent {
    readonly    attribute long      pointerId;
    readonly    attribute double    width;
    readonly    attribute double    height;
    readonly    attribute float     pressure;
    readonly    attribute long      tiltX;
    readonly    attribute long      tiltY;
    readonly    attribute DOMString pointerType;
    readonly    attribute boolean   isPrimary;
};

dictionary PointerEventInit : MouseEventInit {
    long      pointerId = 0;
    double    width = 0;
    double    height = 0;
    float     pressure = 0;
    long      tiltX = 0;
    long      tiltY = 0;
    DOMString pointerType = "";
    boolean   isPrimary = false;
};
