// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Property descriptor of Iterator.prototype.reduce
info: |
  Iterator.prototype.reduce

  * is the initial value of the Iterator.prototype.reduce property of the global object.

  17 ECMAScript Standard Built-in Objects

  every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
features: [globalThis, iterator-helpers]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator.prototype, 'reduce', {
  writable: true,
  enumerable: false,
  configurable: true,
});
