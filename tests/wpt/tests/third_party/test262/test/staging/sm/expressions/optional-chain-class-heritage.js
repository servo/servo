// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Optional expression can be part of a class heritage expression.

var a = {b: null};

class C extends a?.b {}

assert.sameValue(Object.getPrototypeOf(C.prototype), null);

