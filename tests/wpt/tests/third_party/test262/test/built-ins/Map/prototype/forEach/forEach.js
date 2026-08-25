// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Property type and descriptor.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.forEach,
  'function',
  '`typeof Map.prototype.forEach` is `function`'
);

verifyProperty(Map.prototype, 'forEach', {
  writable: true,
  enumerable: false,
  configurable: true,
});
