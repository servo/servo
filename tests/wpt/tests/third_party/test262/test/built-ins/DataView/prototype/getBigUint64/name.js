// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: DataView.prototype.getBigUint64.name property descriptor
includes: [propertyHelper.js]
features: [DataView, ArrayBuffer, BigInt]
---*/

verifyProperty(DataView.prototype.getBigUint64, "name", {
  value: "getBigUint64",
  writable: false,
  enumerable: false,
  configurable: true
});
