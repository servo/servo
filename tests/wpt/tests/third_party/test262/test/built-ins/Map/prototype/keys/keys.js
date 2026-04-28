// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.keys
description: >
  Property type and descriptor.
info: |
  Map.prototype.keys ()

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.keys,
  'function',
  '`typeof Map.prototype.keys` is `function`'
);

verifyProperty(Map.prototype, 'keys', {
  writable: true,
  enumerable: false,
  configurable: true,
});
