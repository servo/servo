// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Property descriptor of Iterator.concat
info: |
  Iterator.concat

  * is the initial value of the Iterator.concat property of the global object.

  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
features: [iterator-sequencing]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator, "concat", {
  value: Iterator.concat,
  writable: true,
  enumerable: false,
  configurable: true,
});
