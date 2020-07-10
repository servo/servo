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

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasState {
  // state
  void save(); // push state on state stack
  void restore(); // pop state stack and restore state
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasTransform {
  // transformations (default transform is the identity matrix)
  void scale(unrestricted double x, unrestricted double y);
  void rotate(unrestricted double angle);
  void translate(unrestricted double x, unrestricted double y);
  void transform(unrestricted double a,
                 unrestricted double b,
                 unrestricted double c,
                 unrestricted double d,
                 unrestricted double e,
                 unrestricted double f);

  [NewObject] DOMMatrix getTransform();
  void setTransform(unrestricted double a,
                    unrestricted double b,
                    unrestricted double c,
                    unrestricted double d,
                    unrestricted double e,
                    unrestricted double f);
  // void setTransform(optional DOMMatrixInit matrix);
  void resetTransform();
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasCompositing {
  // compositing
  attribute unrestricted double globalAlpha; // (default 1.0)
  attribute DOMString globalCompositeOperation; // (default source-over)
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasImageSmoothing {
  // image smoothing
  attribute boolean imageSmoothingEnabled; // (default true)
  // attribute ImageSmoothingQuality imageSmoothingQuality; // (default low)
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasFillStrokeStyles {
  // colours and styles (see also the CanvasDrawingStyles interface)
  attribute (DOMString or CanvasGradient or CanvasPattern) strokeStyle; // (default black)
  attribute (DOMString or CanvasGradient or CanvasPattern) fillStyle; // (default black)
  CanvasGradient createLinearGradient(double x0, double y0, double x1, double y1);
  [Throws]
  CanvasGradient createRadialGradient(double x0, double y0, double r0, double x1, double y1, double r1);
  [Throws]
  CanvasPattern? createPattern(CanvasImageSource image, [TreatNullAs=EmptyString] DOMString repetition);
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasShadowStyles {
  // shadows
  attribute unrestricted double shadowOffsetX; // (default 0)
  attribute unrestricted double shadowOffsetY; // (default 0)
  attribute unrestricted double shadowBlur; // (default 0)
  attribute DOMString shadowColor; // (default transparent black)
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasFilters {
  // filters
  //attribute DOMString filter; // (default "none")
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasRect {
  // rects
  void clearRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  void fillRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  void strokeRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasDrawPath {
  // path API (see also CanvasPath)
  void beginPath();
  void fill(optional CanvasFillRule fillRule = "nonzero");
  //void fill(Path2D path, optional CanvasFillRule fillRule = "nonzero");
  void stroke();
  //void stroke(Path2D path);
  void clip(optional CanvasFillRule fillRule = "nonzero");
  //void clip(Path2D path, optional CanvasFillRule fillRule = "nonzero");
  boolean isPointInPath(unrestricted double x, unrestricted double y,
                        optional CanvasFillRule fillRule = "nonzero");
  //boolean isPointInPath(Path2D path, unrestricted double x, unrestricted double y,
  //                      optional CanvasFillRule fillRule = "nonzero");
  //boolean isPointInStroke(unrestricted double x, unrestricted double y);
  //boolean isPointInStroke(Path2D path, unrestricted double x, unrestricted double y);
};

[Exposed=(PaintWorklet, Window)]
interface mixin CanvasUserInterface {
  //void drawFocusIfNeeded(Element element);
  //void drawFocusIfNeeded(Path2D path, Element element);
  //void scrollPathIntoView();
  //void scrollPathIntoView(Path2D path);
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasText {
  // text (see also the CanvasPathDrawingStyles and CanvasTextDrawingStyles interfaces)
  [Pref="dom.canvas_text.enabled"]
  void fillText(DOMString text, unrestricted double x, unrestricted double y,
                optional unrestricted double maxWidth);
  //void strokeText(DOMString text, unrestricted double x, unrestricted double y,
  //                optional unrestricted double maxWidth);
  [Pref="dom.canvas_text.enabled"]
  TextMetrics measureText(DOMString text);
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasDrawImage {
  // drawing images
  [Throws]
  void drawImage(CanvasImageSource image, unrestricted double dx, unrestricted double dy);
  [Throws]
  void drawImage(CanvasImageSource image, unrestricted double dx, unrestricted double dy,
                                          unrestricted double dw, unrestricted double dh);
  [Throws]
  void drawImage(CanvasImageSource image, unrestricted double sx, unrestricted double sy,
                                          unrestricted double sw, unrestricted double sh,
                                          unrestricted double dx, unrestricted double dy,
                                          unrestricted double dw, unrestricted double dh);
};

[Exposed=(Window, Worker)]
interface mixin CanvasImageData {
  // pixel manipulation
  [Throws]
  ImageData createImageData(long sw, long sh);
  [Throws]
  ImageData createImageData(ImageData imagedata);
  [Throws]
  ImageData getImageData(long sx, long sy, long sw, long sh);
  void putImageData(ImageData imagedata, long dx, long dy);
  void putImageData(ImageData imagedata,
                    long dx, long dy,
                    long dirtyX, long dirtyY,
                    long dirtyWidth, long dirtyHeight);
};

enum CanvasLineCap { "butt", "round", "square" };
enum CanvasLineJoin { "round", "bevel", "miter"};
enum CanvasTextAlign { "start", "end", "left", "right", "center" };
enum CanvasTextBaseline { "top", "hanging", "middle", "alphabetic", "ideographic", "bottom" };
enum CanvasDirection { "ltr", "rtl", "inherit" };

[Exposed=(PaintWorklet, Window, Worker)]
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

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasTextDrawingStyles {
  // text
  attribute DOMString font; // (default 10px sans-serif)
  attribute CanvasTextAlign textAlign; // "start", "end", "left", "right", "center" (default: "start")
  attribute CanvasTextBaseline textBaseline; // "top", "hanging", "middle", "alphabetic",
                                      // "ideographic", "bottom" (default: "alphabetic")
  attribute CanvasDirection direction; // "ltr", "rtl", "inherit" (default: "inherit")
};

[Exposed=(PaintWorklet, Window, Worker)]
interface mixin CanvasPath {
  // shared path API methods
  void closePath();
  void moveTo(unrestricted double x, unrestricted double y);
  void lineTo(unrestricted double x, unrestricted double y);
  void quadraticCurveTo(unrestricted double cpx, unrestricted double cpy,
                        unrestricted double x, unrestricted double y);

  void bezierCurveTo(unrestricted double cp1x,
                     unrestricted double cp1y,
                     unrestricted double cp2x,
                     unrestricted double cp2y,
                     unrestricted double x,
                     unrestricted double y);

  [Throws]
  void arcTo(unrestricted double x1, unrestricted double y1,
             unrestricted double x2, unrestricted double y2,
             unrestricted double radius);

  void rect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);

  [Throws]
  void arc(unrestricted double x, unrestricted double y, unrestricted double radius,
           unrestricted double startAngle, unrestricted double endAngle, optional boolean anticlockwise = false);

  [Throws]
  void ellipse(unrestricted double x, unrestricted double y, unrestricted double radius_x,
               unrestricted double radius_y, unrestricted double rotation, unrestricted double startAngle,
               unrestricted double endAngle, optional boolean anticlockwise = false);
};
