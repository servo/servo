// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: Array.prototype.flatMap.length value and descriptor.
info: >
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [Array.prototype.flatMap]
---*/

assert.sameValue(
  Array.prototype.flatMap.length, 1,
  'The value of `Array.prototype.flatmap.length` is `1`'
);

verifyProperty(Array.prototype.flatMap, 'length', {
  enumerable: false,
  writable: false,
  configurable: true,
});
