// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: Array.prototype.flatmap name value and descriptor.
info: >
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [Array.prototype.flatMap]
---*/

assert.sameValue(
  Array.prototype.flatMap.name, 'flatMap',
  'The value of `Array.prototype.flatMap.name` is `"flatMap"`'
);

verifyProperty(Array.prototype.flatMap, 'name', {
  enumerable: false,
  writable: false,
  configurable: true,
});
