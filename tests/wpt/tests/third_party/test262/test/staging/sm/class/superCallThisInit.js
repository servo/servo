// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function base() { this.prop = 42; }

class testInitialize extends base {
    constructor() {
        assert.throws(ReferenceError, () => this);
        super();
        assert.sameValue(this.prop, 42);
    }
}
assert.sameValue(new testInitialize().prop, 42);

// super() twice is a no-go.
class willThrow extends base {
    constructor() {
        super();
        super();
    }
}
assert.throws(ReferenceError, ()=>new willThrow());

// This is determined at runtime, not the syntax level.
class willStillThrow extends base {
    constructor() {
        for (let i = 0; i < 3; i++) {
            super();
        }
    }
}
assert.throws(ReferenceError, ()=>new willStillThrow());

class canCatchThrow extends base {
    constructor() {
        super();
        try { super(); } catch(e) { }
    }
}
assert.sameValue(new canCatchThrow().prop, 42);

