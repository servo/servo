// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class foo extends null {
    constructor() {
        // Explicit returns of undefined should act the same as falling off the
        // end of the function. That is to say, they should throw.
        return undefined;
    }
}

for (let i = 0; i < 1100; i++)
    assert.throws(ReferenceError, () => new foo());

