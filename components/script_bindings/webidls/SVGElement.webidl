/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://svgwg.org/svg2-draft/types.html#InterfaceSVGElement
[Exposed=Window, Pref="dom.svg.enabled"]
interface SVGElement : Element {

  //[SameObject] readonly attribute SVGAnimatedString className;

  //[SameObject] readonly attribute DOMStringMap dataset;

  //readonly attribute SVGSVGElement? ownerSVGElement;
  //readonly attribute SVGElement? viewportElement;

  //attribute long tabIndex;
  //void focus();
  //void blur();
};

//SVGElement includes GlobalEventHandlers;
//SVGElement includes SVGElementInstance;
SVGElement includes ElementCSSInlineStyle;
SVGElement includes HTMLOrSVGElement;
