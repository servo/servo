// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.keys
description: >
  Array.prototype.keys.name value and descriptor.
info: |
  22.1.3.13 Array.prototype.keys ( )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Array.prototype.keys, "name", {
  value: "keys",
  writable: false,
  enumerable: false,
  configurable: true
});
