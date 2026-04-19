// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise.length is 1.
includes: [propertyHelper.js]
features: [Math.sumPrecise]
---*/

verifyProperty(Math.sumPrecise, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
