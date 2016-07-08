/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#element and
 * https://domparsing.spec.whatwg.org/ and
 * http://dev.w3.org/csswg/cssom-view/ and
 * http://www.w3.org/TR/selectors-api/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Exposed=(Window,Worker)]
interface Element : Node {
  [Constant]
  readonly attribute DOMString? namespaceURI;
  [Constant]
  readonly attribute DOMString? prefix;
  [Constant]
  readonly attribute DOMString localName;
  // Not [Constant] because it depends on which document we're in
  [Pure]
  readonly attribute DOMString tagName;

  [Pure]
           attribute DOMString id;
  [Pure]
           attribute DOMString className;
  [SameObject, PutForwards=value]
  readonly attribute DOMTokenList classList;

  [Pure]
  boolean hasAttributes();
  [SameObject]
  readonly attribute NamedNodeMap attributes;
  [Pure]
  sequence<DOMString> getAttributeNames();
  [Pure]
  DOMString? getAttribute(DOMString name);
  [Pure]
  DOMString? getAttributeNS(DOMString? namespace, DOMString localName);
  [Throws]
  void setAttribute(DOMString name, DOMString value);
  [Throws]
  void setAttributeNS(DOMString? namespace, DOMString name, DOMString value);
  void removeAttribute(DOMString name);
  void removeAttributeNS(DOMString? namespace, DOMString localName);
  boolean hasAttribute(DOMString name);
  boolean hasAttributeNS(DOMString? namespace, DOMString localName);

  [Pure]
  Attr? getAttributeNode(DOMString name);
  [Pure]
  Attr? getAttributeNodeNS(DOMString? namespace, DOMString localName);
  [Throws]
  Attr? setAttributeNode(Attr attr);
  [Throws]
  Attr? setAttributeNodeNS(Attr attr);
  [Throws]
  Attr removeAttributeNode(Attr oldAttr);

  [Pure, Throws]
  Element? closest(DOMString selectors);
  [Pure, Throws]
  boolean matches(DOMString selectors);
  [Pure, Throws]
  boolean webkitMatchesSelector(DOMString selectors); // historical alias of .matches

  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);

  [Throws]
  Element? insertAdjacentElement(DOMString where_, Element element); // historical
  [Throws]
  void insertAdjacentText(DOMString where_, DOMString data);
  [Throws]
  void insertAdjacentHTML(DOMString position, DOMString html);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-element-interface
partial interface Element {
  DOMRectList getClientRects();
  DOMRect getBoundingClientRect();

  [Func="::script_can_initiate_scroll"]
  void scroll(optional ScrollToOptions options);
  [Func="::script_can_initiate_scroll"]
  void scroll(unrestricted double x, unrestricted double y);

  [Func="::script_can_initiate_scroll"]
  void scrollTo(optional ScrollToOptions options);
  [Func="::script_can_initiate_scroll"]
  void scrollTo(unrestricted double x, unrestricted double y);
  [Func="::script_can_initiate_scroll"]
  void scrollBy(optional ScrollToOptions options);
  [Func="::script_can_initiate_scroll"]
  void scrollBy(unrestricted double x, unrestricted double y);
  [Func="::script_can_initiate_scroll"]
  attribute unrestricted double scrollTop;
  [Func="::script_can_initiate_scroll"]
  attribute unrestricted double scrollLeft;
  readonly attribute long scrollWidth;
  readonly attribute long scrollHeight;

  readonly attribute long clientTop;
  readonly attribute long clientLeft;
  readonly attribute long clientWidth;
  readonly attribute long clientHeight;
};

// https://domparsing.spec.whatwg.org/#extensions-to-the-element-interface
partial interface Element {
  [Throws,TreatNullAs=EmptyString]
  attribute DOMString innerHTML;
  [Throws,TreatNullAs=EmptyString]
  attribute DOMString outerHTML;
};

Element implements ChildNode;
Element implements NonDocumentTypeChildNode;
Element implements ParentNode;
