/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://w3c.github.io/touch-events/#idl-def-TouchEvent

// dictionary TouchEventInit : UIEventInit {
//              sequence<Touch> touches = [];
//              sequence<Touch> targetTouches = [];
//              sequence<Touch> changedTouches = [];
//              boolean         altKey;
//              boolean         metaKey;
//              boolean         ctrlKey;
//              boolean         shiftKey;
// };

// [Constructor(DOMString type, optional TouchEventInit eventInitDict)]
interface TouchEvent : UIEvent {
    readonly    attribute TouchList touches;
    readonly    attribute TouchList targetTouches;
    readonly    attribute TouchList changedTouches;
    readonly    attribute boolean   altKey;
    readonly    attribute boolean   metaKey;
    readonly    attribute boolean   ctrlKey;
    readonly    attribute boolean   shiftKey;
};
