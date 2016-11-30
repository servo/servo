/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15
[Constructor(DOMString type, optional WebGLContextEventInit eventInit),
 Exposed=Window]
interface WebGLContextEvent : Event {
    readonly attribute DOMString statusMessage;
};

// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15
dictionary WebGLContextEventInit : EventInit {
    DOMString statusMessage;
};
