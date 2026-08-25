// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  The value of the [[Prototype]] internal slot of Iterator.prototype.flatMap is the
  intrinsic object %FunctionPrototype%.
features: [iterator-helpers]
---*/

assert.sameValue(Object.getPrototypeOf(Iterator.prototype.flatMap), Function.prototype);
