// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The prototype of FinalizationRegistry.prototype is Object.prototype
esid: sec-properties-of-the-finalization-registry-prototype-object
info: |
  The value of the [[Prototype]] internal slot of the FinalizationRegistry prototype object
  is the intrinsic object %ObjectPrototype%.
features: [FinalizationRegistry]
---*/

var proto = Object.getPrototypeOf(FinalizationRegistry.prototype);
assert.sameValue(proto, Object.prototype);
