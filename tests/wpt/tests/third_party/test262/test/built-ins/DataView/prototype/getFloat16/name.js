// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  DataView.prototype.getFloat16.name is "getFloat16".
features: [Float16Array]
includes: [propertyHelper.js]
---*/

verifyProperty(DataView.prototype.getFloat16, "name", {
  value: "getFloat16",
  writable: false,
  enumerable: false,
  configurable: true
});
