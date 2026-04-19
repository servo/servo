// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Setting a "new" prototype to the current [[Prototype]] value should never fail

var x = {}, t = Object.create(x);
Object.preventExtensions(t);
// Should not fail, because it is the same [[Prototype]] value
Object.setPrototypeOf(t, x);

// Object.prototype's [[Prototype]] is immutable, make sure we can still set null
Object.setPrototypeOf(Object.prototype, null);

