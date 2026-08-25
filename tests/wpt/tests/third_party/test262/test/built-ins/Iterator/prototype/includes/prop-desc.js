// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Property descriptor of Iterator.prototype.includes
info: |
  Iterator.prototype.includes

  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 28 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.

includes: [propertyHelper.js]
features: [iterator-includes]
---*/

verifyProperty(Iterator.prototype, 'includes', {
  writable: true,
  enumerable: false,
  configurable: true,
});
