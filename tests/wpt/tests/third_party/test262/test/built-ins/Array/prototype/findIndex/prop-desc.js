// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findindex
description: Property type and descriptor.
info: |
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Array.prototype.findIndex,
  'function',
  '`typeof Array.prototype.findIndex` is `function`'
);

verifyProperty(Array.prototype, "findIndex", {
  writable: true,
  enumerable: false,
  configurable: true
});
