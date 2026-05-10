// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class foo {
    constructor() { }

    method() {
        return (() => eval('super.toString'));
    }
}
assert.sameValue(new foo().method()(), Object.prototype.toString);

