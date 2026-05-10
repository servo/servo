// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-realm-prototype-object
description: >
  The [[Prototype]] of ShadowRealm.prototype is Object.Prototype.
info: |
  Unless otherwise specified every built-in prototype object has the Object prototype
  object, which is the initial value of the expression Object.prototype, as the value
  of its [[Prototype]] internal slot, except the Object prototype object itself.

features: [ShadowRealm]
---*/

assert.sameValue(Object.getPrototypeOf(ShadowRealm.prototype), Object.prototype);
