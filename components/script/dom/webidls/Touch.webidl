/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://w3c.github.io/touch-events/#idl-def-Touch

interface Touch {
    readonly    attribute long        identifier;
    readonly    attribute EventTarget target;
    readonly    attribute double      screenX;
    readonly    attribute double      screenY;
    readonly    attribute double      clientX;
    readonly    attribute double      clientY;
    readonly    attribute double      pageX;
    readonly    attribute double      pageY;
    // readonly    attribute float       radiusX;
    // readonly    attribute float       radiusY;
    // readonly    attribute float       rotationAngle;
    // readonly    attribute float       force;
};
