/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.w3.org/TR/SVG/types.html#InterfaceSVGGeometryElement
[Exposed=Window, Pref="dom_svg_enabled"]
interface SVGGeometryElement : SVGGraphicsElement {
  // [SameObject] readonly attribute SVGAnimatedNumber pathLength;

  // boolean isPointInFill(optional DOMPointInit point);
  // boolean isPointInStroke(optional DOMPointInit point);
  // float getTotalLength();
  // DOMPoint getPointAtLength(float distance);
};
