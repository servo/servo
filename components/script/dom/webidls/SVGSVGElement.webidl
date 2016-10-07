/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://svgwg.org/svg2-draft/struct.html#InterfaceSVGSVGElement
[Pref="dom.svg.enabled"]
interface SVGSVGElement : SVGGraphicsElement {

  //[SameObject] readonly attribute SVGAnimatedLength x;
  //[SameObject] readonly attribute SVGAnimatedLength y;
  //[SameObject] readonly attribute SVGAnimatedLength width;
  //[SameObject] readonly attribute SVGAnimatedLength height;

  //attribute float currentScale;
  //[SameObject] readonly attribute DOMPointReadOnly currentTranslate;

  //NodeList getIntersectionList(DOMRectReadOnly rect, SVGElement? referenceElement);
  //NodeList getEnclosureList(DOMRectReadOnly rect, SVGElement? referenceElement);
  //boolean checkIntersection(SVGElement element, DOMRectReadOnly rect);
  //boolean checkEnclosure(SVGElement element, DOMRectReadOnly rect);

  //void deselectAll();

  //SVGNumber createSVGNumber();
  //SVGLength createSVGLength();
  //SVGAngle createSVGAngle();
  //DOMPoint createSVGPoint();
  //DOMMatrix createSVGMatrix();
  //DOMRect createSVGRect();
  //SVGTransform createSVGTransform();
  //SVGTransform createSVGTransformFromMatrix(DOMMatrixReadOnly matrix);

  //Element getElementById(DOMString elementId);

  // Deprecated methods that have no effect when called,
  // but which are kept for compatibility reasons.
  //unsigned long suspendRedraw(unsigned long maxWaitMilliseconds);
  //void unsuspendRedraw(unsigned long suspendHandleID);
  //void unsuspendRedrawAll();
  //void forceRedraw();
};

//SVGSVGElement implements SVGFitToViewBox;
//SVGSVGElement implements SVGZoomAndPan;
//SVGSVGElement implements WindowEventHandlers;
