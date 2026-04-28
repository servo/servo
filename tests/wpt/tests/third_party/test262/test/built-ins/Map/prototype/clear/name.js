// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
  Map.prototype.entries.name value and descriptor.
info: |
  Map.prototype.clear ( )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Map.prototype.clear, "name", {
  value: "clear",
  writable: false,
  enumerable: false,
  configurable: true
});
