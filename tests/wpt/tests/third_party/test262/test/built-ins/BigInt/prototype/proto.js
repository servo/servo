// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The prototype of BigInt.prototype is Object.prototype
esid: sec-properties-of-the-bigint-prototype-object
info: |
  The value of the [[Prototype]] internal slot of the BigInt prototype object
  is the intrinsic object %ObjectPrototype%.
features: [BigInt]
---*/

var proto = Object.getPrototypeOf(BigInt.prototype);
assert.sameValue(proto, Object.prototype);
