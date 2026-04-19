// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.values
description: >
  Property type and descriptor.
info: |
  Map.prototype.values ()

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.values,
  'function',
  '`typeof Map.prototype.values` is `function`'
);

verifyProperty(Map.prototype, 'values', {
  writable: true,
  enumerable: false,
  configurable: true,
});
