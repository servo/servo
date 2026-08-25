// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Property type and descriptor.
info: |
  WeakMap.prototype.get ( key )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof WeakMap.prototype.get,
  'function',
  '`typeof WeakMap.prototype.get` is `function`'
);

verifyProperty(WeakMap.prototype, 'get', {
  writable: true,
  enumerable: false,
  configurable: true,
});
