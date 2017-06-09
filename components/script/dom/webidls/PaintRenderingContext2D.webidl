/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-paint-api/#paintrenderingcontext2d
[Exposed=PaintWorklet]
interface PaintRenderingContext2D {
};
PaintRenderingContext2D implements CanvasState;
PaintRenderingContext2D implements CanvasTransform;
PaintRenderingContext2D implements CanvasCompositing;
PaintRenderingContext2D implements CanvasImageSmoothing;
PaintRenderingContext2D implements CanvasFillStrokeStyles;
PaintRenderingContext2D implements CanvasShadowStyles;
PaintRenderingContext2D implements CanvasRect;
PaintRenderingContext2D implements CanvasDrawPath;
PaintRenderingContext2D implements CanvasDrawImage;
PaintRenderingContext2D implements CanvasPathDrawingStyles;
PaintRenderingContext2D implements CanvasPath;
