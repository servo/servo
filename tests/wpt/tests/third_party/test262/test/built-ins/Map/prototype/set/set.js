// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Property type and descriptor.
info: |
  Map.prototype.set ( key , value )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.set,
  'function',
  '`typeof Map.prototype.set` is `function`'
);

verifyProperty(Map.prototype, 'set', {
  writable: true,
  enumerable: false,
  configurable: true,
});
