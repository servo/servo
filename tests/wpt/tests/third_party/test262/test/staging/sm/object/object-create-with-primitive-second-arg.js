// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
[1, "", true, Symbol(), undefined].forEach(props => {
    assert.sameValue(Object.getPrototypeOf(Object.create(null, props)), null);
});

assert.throws(TypeError, () => Object.create(null, null));

