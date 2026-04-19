// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base {
    method() { return 1; }
    *gen() { return 2; }
    static sMethod() { return 3; }
    get answer() { return 42; }
}

// Having a default constructor should work, and also not make you lose
// everything for no good reason

assert.sameValue(Object.getPrototypeOf(new base()), base.prototype);
assert.sameValue(new base().method(), 1);
assert.sameValue(new base().gen().next().value, 2);
assert.sameValue(base.sMethod(), 3);
assert.sameValue(new base().answer, 42);

