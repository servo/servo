/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-node
 */

[Abstract, Exposed=(Window,Worker)]
interface Node : EventTarget {
  const unsigned short ELEMENT_NODE = 1;
  const unsigned short ATTRIBUTE_NODE = 2; // historical
  const unsigned short TEXT_NODE = 3;
  const unsigned short CDATA_SECTION_NODE = 4; // historical
  const unsigned short ENTITY_REFERENCE_NODE = 5; // historical
  const unsigned short ENTITY_NODE = 6; // historical
  const unsigned short PROCESSING_INSTRUCTION_NODE = 7;
  const unsigned short COMMENT_NODE = 8;
  const unsigned short DOCUMENT_NODE = 9;
  const unsigned short DOCUMENT_TYPE_NODE = 10;
  const unsigned short DOCUMENT_FRAGMENT_NODE = 11;
  const unsigned short NOTATION_NODE = 12; // historical
  [Constant]
  readonly attribute unsigned short nodeType;
  [Pure]
  readonly attribute DOMString nodeName;

  [Pure]
  readonly attribute USVString baseURI;

  [Pure]
  readonly attribute Document? ownerDocument;

  [Pure]
  Node getRootNode();

  [Pure]
  readonly attribute Node? parentNode;
  [Pure]
  readonly attribute Element? parentElement;
  [Pure]
  boolean hasChildNodes();
  [SameObject]
  readonly attribute NodeList childNodes;
  [Pure]
  readonly attribute Node? firstChild;
  [Pure]
  readonly attribute Node? lastChild;
  [Pure]
  readonly attribute Node? previousSibling;
  [Pure]
  readonly attribute Node? nextSibling;

  [Pure]
           attribute DOMString? nodeValue;
  [Pure]
           attribute DOMString? textContent;
  void normalize();

  Node cloneNode(optional boolean deep = false);
  [Pure]
  boolean isEqualNode(Node? node);
  [Pure]
  boolean isSameNode(Node? otherNode); // historical alias of ===

  const unsigned short DOCUMENT_POSITION_DISCONNECTED = 0x01;
  const unsigned short DOCUMENT_POSITION_PRECEDING = 0x02;
  const unsigned short DOCUMENT_POSITION_FOLLOWING = 0x04;
  const unsigned short DOCUMENT_POSITION_CONTAINS = 0x08;
  const unsigned short DOCUMENT_POSITION_CONTAINED_BY = 0x10;
  const unsigned short DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 0x20;
  [Pure]
  unsigned short compareDocumentPosition(Node other);
  [Pure]
  boolean contains(Node? other);

  [Pure]
  DOMString? lookupPrefix(DOMString? namespace);
  [Pure]
  DOMString? lookupNamespaceURI(DOMString? prefix);
  [Pure]
  boolean isDefaultNamespace(DOMString? namespace);

  [Throws]
  Node insertBefore(Node node, Node? child);
  [Throws]
  Node appendChild(Node node);
  [Throws]
  Node replaceChild(Node node, Node child);
  [Throws]
  Node removeChild(Node child);
};
