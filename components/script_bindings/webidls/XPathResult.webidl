/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-xpathresult
[Exposed=Window, Pref="dom_xpath_enabled"]
interface XPathResult {
  const unsigned short ANY_TYPE = 0;
  const unsigned short NUMBER_TYPE = 1;
  const unsigned short STRING_TYPE = 2;
  const unsigned short BOOLEAN_TYPE = 3;
  const unsigned short UNORDERED_NODE_ITERATOR_TYPE = 4;
  const unsigned short ORDERED_NODE_ITERATOR_TYPE = 5;
  const unsigned short UNORDERED_NODE_SNAPSHOT_TYPE = 6;
  const unsigned short ORDERED_NODE_SNAPSHOT_TYPE = 7;
  const unsigned short ANY_UNORDERED_NODE_TYPE = 8;
  const unsigned short FIRST_ORDERED_NODE_TYPE = 9;

  readonly attribute unsigned short resultType;
  [Throws] readonly attribute unrestricted double numberValue;
  [Throws] readonly attribute DOMString stringValue;
  [Throws] readonly attribute boolean booleanValue;
  [Throws] readonly attribute Node? singleNodeValue;
  [Throws] readonly attribute boolean invalidIteratorState;
  [Throws] readonly attribute unsigned long snapshotLength;

  [Throws] Node? iterateNext();
  [Throws] Node? snapshotItem(unsigned long index);
};
