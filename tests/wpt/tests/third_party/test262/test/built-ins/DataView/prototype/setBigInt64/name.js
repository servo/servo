// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: DataView.prototype.setBigInt64.name property descriptor
includes: [propertyHelper.js]
features: [DataView, ArrayBuffer, BigInt]
---*/

verifyProperty(DataView.prototype.setBigInt64, "name", {
  value: "setBigInt64",
  writable: false,
  enumerable: false,
  configurable: true
});
