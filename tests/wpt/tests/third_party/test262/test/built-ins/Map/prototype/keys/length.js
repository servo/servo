// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.keys
description: >
  Map.prototype.keys.length value and descriptor.
info: |
  Map.prototype.keys ()

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Map.prototype.keys, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
