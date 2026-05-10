// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-math.sumprecise
description: checks the "sumPrecise" property of Math
includes: [propertyHelper.js]
features: [Math.sumPrecise]
---*/

verifyProperty(Math, "sumPrecise", {
  writable: true,
  enumerable: false,
  configurable: true
});
