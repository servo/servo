// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.f16round
description: >
  "f16round" property of Math
features: [Float16Array]
includes: [propertyHelper.js]
---*/

verifyProperty(Math, "f16round", {
  writable: true,
  enumerable: false,
  configurable: true,
});
