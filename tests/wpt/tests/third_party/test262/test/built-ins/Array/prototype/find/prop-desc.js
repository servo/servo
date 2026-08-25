// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: Property type and descriptor.
info: |
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Array.prototype.find,
  'function',
  '`typeof Array.prototype.find` is `function`'
);

verifyProperty(Array.prototype, "find", {
  writable: true,
  enumerable: false,
  configurable: true
});
