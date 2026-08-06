/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#animation-frames
callback FrameRequestCallback = undefined (DOMHighResTimeStamp time);

interface mixin AnimationFrameProvider {
  [Throws] unsigned long requestAnimationFrame(FrameRequestCallback callback);
  [Throws] undefined cancelAnimationFrame(unsigned long handle);
};

Window includes AnimationFrameProvider;
DedicatedWorkerGlobalScope includes AnimationFrameProvider;
