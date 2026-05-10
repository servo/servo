// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base1 {
    constructor() {
        this.base = 1;
    }
}

class base2 {
    constructor() {
        this.base = 2;
    }
}

class inst extends base1 {
    constructor() {
        super();
    }
}

assert.sameValue(new inst().base, 1);

Object.setPrototypeOf(inst, base2);

assert.sameValue(new inst().base, 2);

// Still works with default constructor

class defaultInst extends base1 { }

assert.sameValue(new defaultInst().base, 1);
Object.setPrototypeOf(defaultInst, base2);
assert.sameValue(new defaultInst().base, 2);

