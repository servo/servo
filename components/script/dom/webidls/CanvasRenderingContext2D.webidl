/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#2dcontext

// typedef (HTMLImageElement or
//          SVGImageElement) HTMLOrSVGImageElement;
typedef HTMLImageElement HTMLOrSVGImageElement;

typedef (HTMLOrSVGImageElement or
         /*HTMLVideoElement or*/
         HTMLCanvasElement or
         /*ImageBitmap or*/
         OffscreenCanvas or
         /*VideoFrame or*/
         /*CSSImageValue*/ CSSStyleValue) CanvasImageSource;

enum CanvasFillRule { "nonzero", "evenodd" };

[Exposed=Window]
interface CanvasRenderingContext2D {
  // back-reference to the canvas
  readonly attribute HTMLCanvasElement canvas;
};
CanvasRenderingContext2D includes CanvasState;
CanvasRenderingContext2D includes CanvasTransform;
CanvasRenderingContext2D includes CanvasCompositing;
CanvasRenderingContext2D includes CanvasImageSmoothing;
CanvasRenderingContext2D includes CanvasFillStrokeStyles;
CanvasRenderingContext2D includes CanvasShadowStyles;
CanvasRenderingContext2D includes CanvasFilters;
CanvasRenderingContext2D includes CanvasRect;
CanvasRenderingContext2D includes CanvasDrawPath;
CanvasRenderingContext2D includes CanvasUserInterface;
CanvasRenderingContext2D includes CanvasText;
CanvasRenderingContext2D includes CanvasDrawImage;
CanvasRenderingContext2D includes CanvasImageData;
CanvasRenderingContext2D includes CanvasPathDrawingStyles;
CanvasRenderingContext2D includes CanvasTextDrawingStyles;
CanvasRenderingContext2D includes CanvasPath;

interface mixin CanvasState {
  // state
  undefined save(); // push state on state stack
  undefined restore(); // pop state stack and restore state
  undefined reset();
};

interface mixin CanvasTransform {
  // transformations (default transform is the identity matrix)
  undefined scale(unrestricted double x, unrestricted double y);
  undefined rotate(unrestricted double angle);
  undefined translate(unrestricted double x, unrestricted double y);
  undefined transform(unrestricted double a,
                 unrestricted double b,
                 unrestricted double c,
                 unrestricted double d,
                 unrestricted double e,
                 unrestricted double f);

  [NewObject] DOMMatrix getTransform();
  undefined setTransform(unrestricted double a,
                    unrestricted double b,
                    unrestricted double c,
                    unrestricted double d,
                    unrestricted double e,
                    unrestricted double f);
  // void setTransform(optional DOMMatrixInit matrix);
  undefined resetTransform();
};

interface mixin CanvasCompositing {
  // compositing
  attribute unrestricted double globalAlpha; // (default 1.0)
  attribute DOMString globalCompositeOperation; // (default source-over)
};

interface mixin CanvasImageSmoothing {
  // image smoothing
  attribute boolean imageSmoothingEnabled; // (default true)
  // attribute ImageSmoothingQuality imageSmoothingQuality; // (default low)
};

interface mixin CanvasFillStrokeStyles {
  // colours and styles (see also the CanvasDrawingStyles interface)
  attribute (DOMString or CanvasGradient or CanvasPattern) strokeStyle; // (default black)
  attribute (DOMString or CanvasGradient or CanvasPattern) fillStyle; // (default black)
  CanvasGradient createLinearGradient(double x0, double y0, double x1, double y1);
  [Throws]
  CanvasGradient createRadialGradient(double x0, double y0, double r0, double x1, double y1, double r1);
  [Throws]
  CanvasPattern? createPattern(CanvasImageSource image, [LegacyNullToEmptyString] DOMString repetition);
};

interface mixin CanvasShadowStyles {
  // shadows
  attribute unrestricted double shadowOffsetX; // (default 0)
  attribute unrestricted double shadowOffsetY; // (default 0)
  attribute unrestricted double shadowBlur; // (default 0)
  attribute DOMString shadowColor; // (default transparent black)
};

interface mixin CanvasFilters {
  // filters
  //attribute DOMString filter; // (default "none")
};

interface mixin CanvasRect {
  // rects
  undefined clearRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  undefined fillRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  undefined strokeRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
};

interface mixin CanvasDrawPath {
  // path API (see also CanvasPath)
  undefined beginPath();
  undefined fill(optional CanvasFillRule fillRule = "nonzero");
  //void fill(Path2D path, optional CanvasFillRule fillRule = "nonzero");
  undefined stroke();
  //void stroke(Path2D path);
  undefined clip(optional CanvasFillRule fillRule = "nonzero");
  //void clip(Path2D path, optional CanvasFillRule fillRule = "nonzero");
  boolean isPointInPath(unrestricted double x, unrestricted double y,
                        optional CanvasFillRule fillRule = "nonzero");
  //boolean isPointInPath(Path2D path, unrestricted double x, unrestricted double y,
  //                      optional CanvasFillRule fillRule = "nonzero");
  //boolean isPointInStroke(unrestricted double x, unrestricted double y);
  //boolean isPointInStroke(Path2D path, unrestricted double x, unrestricted double y);
};

interface mixin CanvasUserInterface {
  //void drawFocusIfNeeded(Element element);
  //void drawFocusIfNeeded(Path2D path, Element element);
  //void scrollPathIntoView();
  //void scrollPathIntoView(Path2D path);
};

interface mixin CanvasText {
  // text (see also the CanvasPathDrawingStyles and CanvasTextDrawingStyles interfaces)
  [Pref="dom.canvas_text.enabled"]
  undefined fillText(DOMString text, unrestricted double x, unrestricted double y,
                optional unrestricted double maxWidth);
  //void strokeText(DOMString text, unrestricted double x, unrestricted double y,
  //                optional unrestricted double maxWidth);
  [Pref="dom.canvas_text.enabled"]
  TextMetrics measureText(DOMString text);
};

interface mixin CanvasDrawImage {
  // drawing images
  [Throws]
  undefined drawImage(CanvasImageSource image, unrestricted double dx, unrestricted double dy);
  [Throws]
  undefined drawImage(CanvasImageSource image, unrestricted double dx, unrestricted double dy,
                                          unrestricted double dw, unrestricted double dh);
  [Throws]
  undefined drawImage(CanvasImageSource image, unrestricted double sx, unrestricted double sy,
                                          unrestricted double sw, unrestricted double sh,
                                          unrestricted double dx, unrestricted double dy,
                                          unrestricted double dw, unrestricted double dh);
};

interface mixin CanvasImageData {
  // pixel manipulation
  [Throws]
  ImageData createImageData(long sw, long sh);
  [Throws]
  ImageData createImageData(ImageData imagedata);
  [Throws]
  ImageData getImageData(long sx, long sy, long sw, long sh);
  undefined putImageData(ImageData imagedata, long dx, long dy);
  undefined putImageData(ImageData imagedata,
                    long dx, long dy,
                    long dirtyX, long dirtyY,
                    long dirtyWidth, long dirtyHeight);
};

enum CanvasLineCap { "butt", "round", "square" };
enum CanvasLineJoin { "round", "bevel", "miter"};
enum CanvasTextAlign { "start", "end", "left", "right", "center" };
enum CanvasTextBaseline { "top", "hanging", "middle", "alphabetic", "ideographic", "bottom" };
enum CanvasDirection { "ltr", "rtl", "inherit" };

interface mixin CanvasPathDrawingStyles {
  // line caps/joins
  attribute unrestricted double lineWidth; // (default 1)
  attribute CanvasLineCap lineCap; // (default "butt")
  attribute CanvasLineJoin lineJoin; // (default "miter")
  attribute unrestricted double miterLimit; // (default 10)

  // dashed lines
  //void setLineDash(sequence<unrestricted double> segments); // default empty
  //sequence<unrestricted double> getLineDash();
  //attribute unrestricted double lineDashOffset;
};

interface mixin CanvasTextDrawingStyles {
  // text
  attribute DOMString font; // (default 10px sans-serif)
  attribute CanvasTextAlign textAlign; // "start", "end", "left", "right", "center" (default: "start")
  attribute CanvasTextBaseline textBaseline; // "top", "hanging", "middle", "alphabetic",
                                      // "ideographic", "bottom" (default: "alphabetic")
  attribute CanvasDirection direction; // "ltr", "rtl", "inherit" (default: "inherit")
};

interface mixin CanvasPath {
  // shared path API methods
  undefined closePath();
  undefined moveTo(unrestricted double x, unrestricted double y);
  undefined lineTo(unrestricted double x, unrestricted double y);
  undefined quadraticCurveTo(unrestricted double cpx, unrestricted double cpy,
                        unrestricted double x, unrestricted double y);

  undefined bezierCurveTo(unrestricted double cp1x,
                     unrestricted double cp1y,
                     unrestricted double cp2x,
                     unrestricted double cp2y,
                     unrestricted double x,
                     unrestricted double y);

  [Throws]
  undefined arcTo(unrestricted double x1, unrestricted double y1,
             unrestricted double x2, unrestricted double y2,
             unrestricted double radius);

  undefined rect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);

  [Throws]
  undefined arc(unrestricted double x, unrestricted double y, unrestricted double radius,
           unrestricted double startAngle, unrestricted double endAngle, optional boolean anticlockwise = false);

  [Throws]
  undefined ellipse(unrestricted double x, unrestricted double y, unrestricted double radius_x,
               unrestricted double radius_y, unrestricted double rotation, unrestricted double startAngle,
               unrestricted double endAngle, optional boolean anticlockwise = false);
};

[Exposed=(Window, PaintWorklet, Worker)]
interface CanvasGradient {
  // opaque object
  [Throws]
  undefined addColorStop(double offset, DOMString color);
};

[Exposed=(Window, PaintWorklet, Worker)]
interface CanvasPattern {
  // opaque object
  //undefined setTransform(optional DOMMatrix2DInit transform = {});
};

[Exposed=(Window,Worker),
 Serializable]
interface ImageData {
  [Throws] constructor(unsigned long sw, unsigned long sh/*, optional ImageDataSettings settings = {}*/);
  [Throws] constructor(/* Uint8ClampedArray */ object data, unsigned long sw, optional unsigned long sh
              /*, optional ImageDataSettings settings = {}*/);

  readonly attribute unsigned long width;
  readonly attribute unsigned long height;
  [Throws] readonly attribute Uint8ClampedArray data;
  //readonly attribute PredefinedColorSpace colorSpace;
};
