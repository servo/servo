// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class foo extends null {
    constructor() {
        // Let it fall off the edge and throw.
    }
}

for (let i = 0; i < 1100; i++)
    assert.throws(ReferenceError, () => new foo());

