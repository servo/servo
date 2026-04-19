// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Map.prototype.set.length value and descriptor.
info: |
  Map.prototype.set ( key , value )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Map.prototype.set, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
