/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-paint-api/#paintrenderingcontext2d
[Pref="dom.worklet.enabled", Exposed=PaintWorklet]
interface PaintRenderingContext2D {
};
PaintRenderingContext2D includes CanvasState;
PaintRenderingContext2D includes CanvasTransform;
PaintRenderingContext2D includes CanvasCompositing;
PaintRenderingContext2D includes CanvasImageSmoothing;
PaintRenderingContext2D includes CanvasFillStrokeStyles;
PaintRenderingContext2D includes CanvasShadowStyles;
PaintRenderingContext2D includes CanvasRect;
PaintRenderingContext2D includes CanvasDrawPath;
PaintRenderingContext2D includes CanvasDrawImage;
PaintRenderingContext2D includes CanvasPathDrawingStyles;
PaintRenderingContext2D includes CanvasPath;
