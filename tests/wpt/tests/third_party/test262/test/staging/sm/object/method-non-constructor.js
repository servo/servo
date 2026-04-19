// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var obj = { method() { } };
assert.throws(TypeError, () => {
    new obj.method;
});

obj = { constructor() { } };
assert.throws(TypeError, () => {
    new obj.constructor;
});

