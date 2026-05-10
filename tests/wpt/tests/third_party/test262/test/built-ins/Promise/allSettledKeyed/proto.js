// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  The value of the [[Prototype]] internal slot of Promise.allSettledKeyed is the
  intrinsic object %FunctionPrototype%.
features: [await-dictionary]
---*/

assert.sameValue(
  Object.getPrototypeOf(Promise.allSettledKeyed),
  Function.prototype,
  "Object.getPrototypeOf(Promise.allSettledKeyed) must return the value of Function.prototype"
);
