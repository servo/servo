// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class foo extends null {
    constructor() {
        // If you return an object, we don't care that |this| went
        // uninitialized
        return {};
    }
}

for (let i = 0; i < 1100; i++)
    new foo();

