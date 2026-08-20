/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/uievents/#legacy-textevent-events
[Exposed=Window]
interface TextEvent : UIEvent {
    readonly attribute DOMString data;
    undefined initTextEvent(DOMString type,
        optional boolean bubbles = false,
        optional boolean cancelable = false,
        optional Window? view = null,
        optional DOMString data = "undefined");
};
