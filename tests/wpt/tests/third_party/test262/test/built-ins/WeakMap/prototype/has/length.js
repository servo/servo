// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  WeakMap.prototype.has.length value and writability.
info: |
  WeakMap.prototype.has ( value )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(WeakMap.prototype.has, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
