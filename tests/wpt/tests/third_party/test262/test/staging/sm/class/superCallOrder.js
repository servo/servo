// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function base() { }

class beforeSwizzle extends base {
    constructor() {
        super(Object.setPrototypeOf(beforeSwizzle, null));
    }
}

new beforeSwizzle();

function MyError() {}

// Again, testing both dynamic prototype dispatch, and that we verify the function
// is a constructor after evaluating args
class beforeThrow extends base {
    constructor() {
        function thrower() { throw new MyError(); }
        super(thrower());
    }
}

Object.setPrototypeOf(beforeThrow, Math.sin);

// Won't throw that Math.sin is not a constructor before evaluating the args
assert.throws(MyError, () => new beforeThrow());

