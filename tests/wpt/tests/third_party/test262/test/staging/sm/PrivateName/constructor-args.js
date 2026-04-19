// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class A {
    #x = "hello";
    constructor(o = this.#x) {
        this.value = o;
    }
};

var a = new A;
assert.sameValue(a.value, "hello");


class B extends A {
    constructor() {
        // Cannot access 'this' until super() called.
        super();
        assert.sameValue("value" in this, true);
        assert.sameValue(this.value, "hello");
    }
}

var b = new B;

