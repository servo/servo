// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allkeyed
description: >
  The value of the [[Prototype]] internal slot of Promise.allKeyed is the
  intrinsic object %FunctionPrototype%.
features: [await-dictionary]
---*/

assert.sameValue(
  Object.getPrototypeOf(Promise.allKeyed),
  Function.prototype,
  "Object.getPrototypeOf(Promise.allKeyed) must return the value of Function.prototype"
);
