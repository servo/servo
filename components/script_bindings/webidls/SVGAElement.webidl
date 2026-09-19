/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://svgwg.org/svg2-draft/linking.html#InterfaceSVGAElement
[Exposed=Window]
interface SVGAElement : SVGGraphicsElement {
  //[SameObject] readonly attribute SVGAnimatedString target;
  //attribute DOMString download;
  //attribute USVString ping;
  //attribute DOMString rel;
  //[SameObject, PutForwards=value] readonly attribute DOMTokenList relList;
  //attribute DOMString hreflang;
  //attribute DOMString type;

  //attribute DOMString referrerPolicy;
};

// SVGAElement includes SVGURIReference;
// SVGAElement includes HTMLHyperlinkElementUtils;
