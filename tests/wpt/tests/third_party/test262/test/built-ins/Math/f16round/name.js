// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.f16round
description: >
  Math.f16round.name is "f16round"
features: [Float16Array]
includes: [propertyHelper.js]
---*/

verifyProperty(Math.f16round, 'name', {
  value: 'f16round',
  enumerable: false,
  writable: false,
  configurable: true
});
