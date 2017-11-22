/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface
//[Pref="dom.offscreen_canvas.enabled"]

typedef (OffscreenCanvasRenderingContext2D or
        WebGLRenderingContext) OffscreenRenderingContext;

dictionary ImageEncodeOptions {
  DOMString type = "image/png";
  unrestricted double quality = 1.0;
};

//enum OffscreenRenderingContextType { "2d", "webgl" };

[Pref="dom.offscreen_canvas.enabled", Constructor([EnforceRange] unsigned long long width, [EnforceRange] unsigned long long height), Exposed=(Window,Worker)]
interface OffscreenCanvas : EventTarget {
  attribute unsigned long long width;
  attribute unsigned long long height;
};
//  OffscreenRenderingContext? getContext(OffscreenRenderingContextType contextType, any... arguments);
