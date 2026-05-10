// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure we get the proper side effects.
// |delete super.prop| and |delete super[expr]| throw universally.

class base {
    constructor() { }
}

class derived extends base {
    constructor() { super(); }
    testDeleteProp() { delete super.prop; }
    testDeleteElem() {
        let sideEffect = 0;
        assert.throws(ReferenceError, () => delete super[sideEffect = 1]);
        assert.sameValue(sideEffect, 1);
    }
}

var d = new derived();
assert.throws(ReferenceError, () => d.testDeleteProp());
d.testDeleteElem();

// |delete super.x| does not delete anything before throwing.
var thing1 = {
    go() { delete super.toString; }
};
let saved = Object.prototype.toString;
assert.throws(ReferenceError, () => thing1.go());
assert.sameValue(Object.prototype.toString, saved);

// |delete super.x| does not tell the prototype to delete anything, when it's a proxy.
var thing2 = {
    go() { delete super.prop; }
};
Object.setPrototypeOf(thing2, new Proxy({}, {
    deleteProperty(x) { throw "FAIL"; }
}));
assert.throws(ReferenceError, () => thing2.go());

class derivedTestDeleteProp extends base {
    constructor() {
        // The deletion error is a reference error, even after munging the prototype
        // chain.
        Object.setPrototypeOf(derivedTestDeleteProp.prototype, null);

        assert.throws(ReferenceError, () => delete super.prop);

        super();

        assert.throws(ReferenceError, () => delete super.prop);

        return {};
    }
}

new derivedTestDeleteProp();

