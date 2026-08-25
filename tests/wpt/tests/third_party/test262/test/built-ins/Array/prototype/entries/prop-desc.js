// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.entries
description: >
  Property type and descriptor.
info: |
  22.1.3.4 Array.prototype.entries ( )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Array.prototype.entries,
  'function',
  '`typeof Array.prototype.entries` is `function`'
);

verifyProperty(Array.prototype, "entries", {
  writable: true,
  enumerable: false,
  configurable: true
});
