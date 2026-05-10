// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// The cycle check in 9.1.2 [[SetPrototypeOf]] prevents cross-realm cycles
// involving only ordinary objects.

var gw = $262.createRealm().global;

var obj = {};
var w = gw.Object.create(obj);
assert.throws(TypeError, () => Object.setPrototypeOf(obj, w));
assert.throws(gw.TypeError, () => gw.Object.setPrototypeOf(obj, w));

