// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Make sure it doesn't matter when we make the arrow function
new class extends class { } {
    constructor() {
        let arrow = () => this;
        assert.throws(ReferenceError, arrow);
        super();
        assert.sameValue(arrow(), this);
    }
}();

