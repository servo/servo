/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15
[Exposed=Window]
interface WebGLContextEvent : Event {
    [Throws] constructor(DOMString type, optional WebGLContextEventInit eventInit = {});
    readonly attribute DOMString statusMessage;
};

// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15
dictionary WebGLContextEventInit : EventInit {
    DOMString statusMessage;
};
