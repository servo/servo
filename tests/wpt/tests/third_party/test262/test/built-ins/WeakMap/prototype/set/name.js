// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: WeakMap.prototype.set.name descriptor
info: |
  WeakMap.prototype.set ( value )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(WeakMap.prototype.set, "name", {
  value: "set",
  writable: false,
  enumerable: false,
  configurable: true
});
