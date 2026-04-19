// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The prototype of DisposableStack.prototype is Object.prototype
esid: sec-properties-of-the-disposablestack-prototype-object
info: |
  The value of the [[Prototype]] internal slot of the DisposableStack prototype object
  is the intrinsic object %Object.prototype%.
features: [explicit-resource-management]
---*/

var proto = Object.getPrototypeOf(DisposableStack.prototype);
assert.sameValue(proto, Object.prototype);
