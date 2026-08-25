// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/

var otherGlobal = $262.createRealm().global;

for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.includes.length, 1);

    assert.sameValue(new constructor([1, 2, 3]).includes(1), true);
    assert.sameValue(new constructor([1, 2, 3]).includes(2), true);
    assert.sameValue(new constructor([1, 2, 3]).includes(3), true);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, 1), true);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, -2), true);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, -100), true);

    assert.sameValue(new constructor([1, 2, 3]).includes("2"), false);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, 2), false);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, -1), false);
    assert.sameValue(new constructor([1, 2, 3]).includes(2, 100), false);

    // Called from other globals.
    var includes = otherGlobal[constructor.name].prototype.includes;
    assert.sameValue(includes.call(new constructor([1, 2, 3]), 2), true);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.includes.call(invalidReceiver);
        }, "Assert that reverse fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).includes(2), true);
}

