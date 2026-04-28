// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

new class extends class { } {
    constructor() {
        let a1 = () => this;
        let a2 = (() => super());
        assert.throws(ReferenceError, a1);
        assert.sameValue(a2(), a1());
    }
}();

