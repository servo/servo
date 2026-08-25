// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
/* Make sure that the default derived class constructor has the required spread semantics.
 *
 * Test credit André Bargull
 */

// <https://github.com/tc39/ecma262/pull/2216> changed default derived class
// constructors to no longer execute the spread iteration protocol.
Array.prototype[Symbol.iterator] = function*() {
    throw new Error("unexpected call");
};

class Base {
    constructor(a, b) {
        assert.sameValue(a, 1);
        assert.sameValue(b, 2);
    }
};
class Derived extends Base {};

new Derived(1, 2);

