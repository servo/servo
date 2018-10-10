/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#2dcontext

// typedef (HTMLImageElement or
//          SVGImageElement) HTMLOrSVGImageElement;
typedef HTMLImageElement HTMLOrSVGImageElement;

typedef (HTMLOrSVGImageElement or
         /*HTMLVideoElement or*/
         HTMLCanvasElement or
         /*ImageBitmap or*/
         /*OffscreenCanvas or*/
         /*CSSImageValue*/ CSSStyleValue) CanvasImageSource;

enum CanvasFillRule { "nonzero", "evenodd" };

[Exposed=Window]
interface CanvasRenderingContext2D {
  // back-reference to the canvas
  readonly attribute HTMLCanvasElement canvas;
};
CanvasRenderingContext2D implements CanvasState;
CanvasRenderingContext2D implements CanvasTransform;
CanvasRenderingContext2D implements CanvasCompositing;
CanvasRenderingContext2D implements CanvasImageSmoothing;
CanvasRenderingContext2D implements CanvasFillStrokeStyles;
CanvasRenderingContext2D implements CanvasShadowStyles;
CanvasRenderingContext2D implements CanvasFilters;
CanvasRenderingContext2D implements CanvasRect;
CanvasRenderingContext2D implements CanvasDrawPath;
CanvasRenderingContext2D implements CanvasUserInterface;
CanvasRenderingContext2D implements CanvasText;
CanvasRenderingContext2D implements CanvasDrawImage;
CanvasRenderingContext2D implements CanvasImageData;
CanvasRenderingContext2D implements CanvasPathDrawingStyles;
CanvasRenderingContext2D implements CanvasTextDrawingStyles;
CanvasRenderingContext2D implements CanvasPath;

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasState {
  // state
  void save(); // push state on state stack
  void restore(); // pop state stack and restore state
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasTransform {
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

  // [NewObject] DOMMatrix getTransform();
  void setTransform(unrestricted double a,
                    unrestricted double b,
                    unrestricted double c,
                    unrestricted double d,
                    unrestricted double e,
                    unrestricted double f);
  // void setTransform(optional DOMMatrixInit matrix);
  void resetTransform();
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasCompositing {
  // compositing
  attribute unrestricted double globalAlpha; // (default 1.0)
  attribute DOMString globalCompositeOperation; // (default source-over)
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasImageSmoothing {
  // image smoothing
  attribute boolean imageSmoothingEnabled; // (default true)
  // attribute ImageSmoothingQuality imageSmoothingQuality; // (default low)
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasFillStrokeStyles {
  // colours and styles (see also the CanvasDrawingStyles interface)
  attribute (DOMString or CanvasGradient or CanvasPattern) strokeStyle; // (default black)
  attribute (DOMString or CanvasGradient or CanvasPattern) fillStyle; // (default black)
  CanvasGradient createLinearGradient(double x0, double y0, double x1, double y1);
  [Throws]
  CanvasGradient createRadialGradient(double x0, double y0, double r0, double x1, double y1, double r1);
  [Throws]
  CanvasPattern createPattern(CanvasImageSource image, [TreatNullAs=EmptyString] DOMString repetition);
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasShadowStyles {
  // shadows
  attribute unrestricted double shadowOffsetX; // (default 0)
  attribute unrestricted double shadowOffsetY; // (default 0)
  attribute unrestricted double shadowBlur; // (default 0)
  attribute DOMString shadowColor; // (default transparent black)
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasFilters {
  // filters
  //attribute DOMString filter; // (default "none")
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasRect {
  // rects
  void clearRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  void fillRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
  void strokeRect(unrestricted double x, unrestricted double y, unrestricted double w, unrestricted double h);
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasDrawPath {
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

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasUserInterface {
  //void drawFocusIfNeeded(Element element);
  //void drawFocusIfNeeded(Path2D path, Element element);
  //void scrollPathIntoView();
  //void scrollPathIntoView(Path2D path);
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasText {
  // text (see also the CanvasPathDrawingStyles and CanvasTextDrawingStyles interfaces)
  [Pref="dom.canvas-text.enabled"]
  void fillText(DOMString text, unrestricted double x, unrestricted double y,
                optional unrestricted double maxWidth);
  //void strokeText(DOMString text, unrestricted double x, unrestricted double y,
  //                optional unrestricted double maxWidth);
  //TextMetrics measureText(DOMString text);
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasDrawImage {
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

[Exposed=Window, NoInterfaceObject]
interface CanvasImageData {
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

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasPathDrawingStyles {
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

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasTextDrawingStyles {
  // text
  //attribute DOMString font; // (default 10px sans-serif)
  //attribute CanvasTextAlign textAlign; // "start", "end", "left", "right", "center" (default: "start")
  //attribute CanvasTextBaseline textBaseline; // "top", "hanging", "middle", "alphabetic",
                                      // "ideographic", "bottom" (default: "alphabetic")
  //attribute CanvasDirection direction; // "ltr", "rtl", "inherit" (default: "inherit")
};

[Exposed=(PaintWorklet, Window), NoInterfaceObject]
interface CanvasPath {
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
