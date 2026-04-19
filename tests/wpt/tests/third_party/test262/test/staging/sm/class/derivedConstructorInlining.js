// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class foo extends null {
    constructor() {
        // Anything that tests |this| should throw, so just let it run off the
        // end.
    }
}

function intermediate() {
    new foo();
}

for (let i = 0; i < 1100; i++)
    assert.throws(ReferenceError, intermediate);

