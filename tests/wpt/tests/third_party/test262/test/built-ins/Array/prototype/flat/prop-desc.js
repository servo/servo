// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: Property type and descriptor.
info: >
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [Array.prototype.flat]
---*/

assert.sameValue(
  typeof Array.prototype.flat,
  'function',
  '`typeof Array.prototype.flat` is `function`'
);

verifyProperty(Array.prototype, 'flat', {
  enumerable: false,
  writable: true,
  configurable: true,
});
