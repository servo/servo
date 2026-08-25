// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// While |super| is common in classes, it also works in object litterals,
// where there is no guarantee of strict mode. Check that we do not somehow
// get strict mode semantics when they were not called for

// |undefined|, writable: false
Object.defineProperty(Object.prototype, "prop", { writable: false });

class strictAssignmentTest {
    constructor() {
        // Strict mode. Throws.
        super.prop = 14;
    }
}

assert.throws(TypeError, ()=>new strictAssignmentTest());

// Non-strict. Silent failure.
({ test() { super.prop = 14; } }).test();

assert.sameValue(Object.prototype.prop, undefined);

