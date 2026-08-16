// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes has a "length" property whose value is 1.
info: |
  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in
  Function object has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [iterator-includes]
---*/

verifyProperty(Iterator.prototype.includes, 'length', {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
