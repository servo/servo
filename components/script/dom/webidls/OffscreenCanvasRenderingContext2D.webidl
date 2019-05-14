/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-offscreen-2d-rendering-context
[Exposed=(Window,Worker), Pref="dom.offscreen_canvas.enabled"]
interface OffscreenCanvasRenderingContext2D {
  //void commit();
  readonly attribute OffscreenCanvas canvas;
};
OffscreenCanvasRenderingContext2D implements CanvasState;
OffscreenCanvasRenderingContext2D implements CanvasCompositing;
OffscreenCanvasRenderingContext2D implements CanvasImageSmoothing;
OffscreenCanvasRenderingContext2D implements CanvasFillStrokeStyles;
OffscreenCanvasRenderingContext2D implements CanvasShadowStyles;
OffscreenCanvasRenderingContext2D implements CanvasFilters;
OffscreenCanvasRenderingContext2D implements CanvasRect;

//OffscreenCanvasRenderingContext2D includes CanvasTransform;
//OffscreenCanvasRenderingContext2D includes CanvasDrawPath;
OffscreenCanvasRenderingContext2D implements CanvasText;
//OffscreenCanvasRenderingContext2D includes CanvasDrawImage;
//OffscreenCanvasRenderingContext2D includes CanvasImageData;
//OffscreenCanvasRenderingContext2D includes CanvasPathDrawingStyles;
//OffscreenCanvasRenderingContext2D includes CanvasTextDrawingStyles;
//OffscreenCanvasRenderingContext2D includes CanvasPath;




