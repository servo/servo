// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.delete
description: >
    Map.prototype.delete ( )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Map.prototype.delete,
  'function',
  'typeof Map.prototype.delete is "function"'
);

verifyProperty(Map.prototype, 'delete', {
  writable: true,
  enumerable: false,
  configurable: true,
});
