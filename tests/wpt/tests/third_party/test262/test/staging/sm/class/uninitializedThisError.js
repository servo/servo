// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class TestNormal extends class {} {
    constructor() { this; }
}
assert.throws(ReferenceError, () => new TestNormal());

class TestEval extends class {} {
    constructor() { eval("this") }
}
assert.throws(ReferenceError, () => new TestEval());

class TestNestedEval extends class {} {
    constructor() { eval("eval('this')") }
}
assert.throws(ReferenceError, () => new TestNestedEval());

assert.throws(ReferenceError, () => {
    new class extends class {} {
        constructor() { eval("this") }
    }
});

class TestArrow extends class {} {
    constructor() { (() => this)(); }
}
assert.throws(ReferenceError, () => new TestArrow());

class TestArrowEval extends class {} {
    constructor() { (() => eval("this"))(); }
}
assert.throws(ReferenceError, () => new TestArrowEval());

class TestEvalArrow extends class {} {
    constructor() { eval("(() => this)()"); }
}
assert.throws(ReferenceError, () => new TestEvalArrow());

class TestTypeOf extends class {} {
    constructor() { eval("typeof this"); }
}
assert.throws(ReferenceError, () => new TestTypeOf());
