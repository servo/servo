// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise does not implement [[Construct]], is not new-able
includes: [isConstructor.js]
features: [Reflect.construct, Math.sumPrecise]
---*/

assert.sameValue(isConstructor(Math.sumPrecise), false, "isConstructor(Math.sumPrecise) must return false");

assert.throws(TypeError, function () {
  new Math.sumPrecise();
});
