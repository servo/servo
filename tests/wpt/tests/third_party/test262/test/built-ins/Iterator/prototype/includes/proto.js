// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  The value of the [[Prototype]] internal slot of Iterator.prototype.includes is
  the intrinsic object %Function.prototype%.
features: [iterator-includes]
---*/

assert.sameValue(Object.getPrototypeOf(Iterator.prototype.includes), Function.prototype);
