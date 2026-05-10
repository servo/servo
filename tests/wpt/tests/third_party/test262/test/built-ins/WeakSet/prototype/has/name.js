// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  WeakSet.prototype.has.name value and writability.
info: |
  WeakSet.prototype.has ( value )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(WeakSet.prototype.has, "name", {
  value: "has",
  writable: false,
  enumerable: false,
  configurable: true
});
