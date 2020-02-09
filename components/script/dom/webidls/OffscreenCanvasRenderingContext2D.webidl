/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-offscreen-2d-rendering-context
[Exposed=(Window,Worker), Pref="dom.offscreen_canvas.enabled"]
interface OffscreenCanvasRenderingContext2D {
  //void commit();
  readonly attribute OffscreenCanvas canvas;
};
OffscreenCanvasRenderingContext2D includes CanvasState;
OffscreenCanvasRenderingContext2D includes CanvasCompositing;
OffscreenCanvasRenderingContext2D includes CanvasImageSmoothing;
OffscreenCanvasRenderingContext2D includes CanvasFillStrokeStyles;
OffscreenCanvasRenderingContext2D includes CanvasShadowStyles;
OffscreenCanvasRenderingContext2D includes CanvasFilters;
OffscreenCanvasRenderingContext2D includes CanvasRect;

OffscreenCanvasRenderingContext2D includes CanvasTransform;
OffscreenCanvasRenderingContext2D includes CanvasDrawPath;
OffscreenCanvasRenderingContext2D includes CanvasText;
OffscreenCanvasRenderingContext2D includes CanvasDrawImage;
OffscreenCanvasRenderingContext2D includes CanvasImageData;
OffscreenCanvasRenderingContext2D includes CanvasPathDrawingStyles;
OffscreenCanvasRenderingContext2D includes CanvasTextDrawingStyles;
OffscreenCanvasRenderingContext2D includes CanvasPath;




