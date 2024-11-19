/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#element and
 * https://w3c.github.io/DOM-Parsing/ and
 * http://dev.w3.org/csswg/cssom-view/ and
 * http://www.w3.org/TR/selectors-api/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Exposed=Window]
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

  [CEReactions, Pure]
           attribute DOMString id;
  [CEReactions, Pure]
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
  [CEReactions, Throws]
  boolean toggleAttribute(DOMString name, optional boolean force);
  [CEReactions, Throws]
  undefined setAttribute(DOMString name, DOMString value);
  [CEReactions, Throws]
  undefined setAttributeNS(DOMString? namespace, DOMString name, DOMString value);
  [CEReactions]
  undefined removeAttribute(DOMString name);
  [CEReactions]
  undefined removeAttributeNS(DOMString? namespace, DOMString localName);
  boolean hasAttribute(DOMString name);
  boolean hasAttributeNS(DOMString? namespace, DOMString localName);

  [Pure]
  Attr? getAttributeNode(DOMString name);
  [Pure]
  Attr? getAttributeNodeNS(DOMString? namespace, DOMString localName);
  [CEReactions, Throws]
  Attr? setAttributeNode(Attr attr);
  [CEReactions, Throws]
  Attr? setAttributeNodeNS(Attr attr);
  [CEReactions, Throws]
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

  [CEReactions, Throws]
  Element? insertAdjacentElement(DOMString where_, Element element); // historical
  [Throws]
  undefined insertAdjacentText(DOMString where_, DOMString data);
  [CEReactions, Throws]
  undefined insertAdjacentHTML(DOMString position, DOMString html);

  [Throws, Pref="dom.shadowdom.enabled"] ShadowRoot attachShadow(ShadowRootInit init);
  readonly attribute ShadowRoot? shadowRoot;
};

dictionary ShadowRootInit {
  required ShadowRootMode mode;
  // boolean delegatesFocus = false;
  // SlotAssignmentMode slotAssignment = "named";
  // boolean clonable = false;
  // boolean serializable = false;
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-element-interface
partial interface Element {
  DOMRectList getClientRects();
  [NewObject]
  DOMRect getBoundingClientRect();

  undefined scroll(optional ScrollToOptions options = {});
  undefined scroll(unrestricted double x, unrestricted double y);

  undefined scrollTo(optional ScrollToOptions options = {});
  undefined scrollTo(unrestricted double x, unrestricted double y);
  undefined scrollBy(optional ScrollToOptions options = {});
  undefined scrollBy(unrestricted double x, unrestricted double y);
  attribute unrestricted double scrollTop;
  attribute unrestricted double scrollLeft;
  readonly attribute long scrollWidth;
  readonly attribute long scrollHeight;

  readonly attribute long clientTop;
  readonly attribute long clientLeft;
  readonly attribute long clientWidth;
  readonly attribute long clientHeight;
};

// https://w3c.github.io/DOM-Parsing/#extensions-to-the-element-interface
partial interface Element {
  [CEReactions, Throws]
  attribute [LegacyNullToEmptyString] DOMString innerHTML;
  [CEReactions, Throws]
  attribute [LegacyNullToEmptyString] DOMString outerHTML;
};

// https://fullscreen.spec.whatwg.org/#api
partial interface Element {
  Promise<undefined> requestFullscreen();
};

Element includes ChildNode;
Element includes NonDocumentTypeChildNode;
Element includes ParentNode;
Element includes ActivatableElement;
Element includes ARIAMixin;
