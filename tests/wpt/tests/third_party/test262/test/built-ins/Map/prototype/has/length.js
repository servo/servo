// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.has
description: >
  Map.prototype.has.length value and descriptor.
info: |
  Map.prototype.has ( key )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Map.prototype.has, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
