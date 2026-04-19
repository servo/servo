// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise.name is "sumPrecise".
includes: [propertyHelper.js]
features: [Math.sumPrecise]
---*/

verifyProperty(Math.sumPrecise, "name", {
  value: "sumPrecise",
  writable: false,
  enumerable: false,
  configurable: true
});
