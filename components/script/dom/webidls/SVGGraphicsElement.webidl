/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://svgwg.org/svg2-draft/types.html#InterfaceSVGGraphicsElement
//dictionary SVGBoundingBoxOptions {
//  boolean fill = true;
//  boolean stroke = false;
//  boolean markers = false;
//  boolean clipped = false;
//};

[Abstract, Pref="dom.svg.enabled"]
interface SVGGraphicsElement : SVGElement {
  //[SameObject] readonly attribute SVGAnimatedTransformList transform;

  //DOMRect getBBox(optional SVGBoundingBoxOptions options);
  //DOMMatrix? getCTM();
  //DOMMatrix? getScreenCTM();
};

//SVGGraphicsElement implements SVGTests;
