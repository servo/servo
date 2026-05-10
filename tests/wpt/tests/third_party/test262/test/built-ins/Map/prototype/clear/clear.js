// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
    Map.prototype.clear ( )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.clear,
  'function',
  'typeof Map.prototype.clear is "function"'
);

verifyProperty(Map.prototype, 'clear', {
  writable: true,
  enumerable: false,
  configurable: true,
});
