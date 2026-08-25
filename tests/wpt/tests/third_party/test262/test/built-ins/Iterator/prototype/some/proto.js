// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  The value of the [[Prototype]] internal slot of Iterator.prototype.some is the
  intrinsic object %Function%.
features: [iterator-helpers]
---*/

assert.sameValue(Object.getPrototypeOf(Iterator.prototype.some), Function.prototype);
