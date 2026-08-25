// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

var otherGlobal = $262.createRealm().global;

for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.join.length, 1);

    assert.sameValue(new constructor([1, 2, 3]).join(), "1,2,3");
    assert.sameValue(new constructor([1, 2, 3]).join(undefined), "1,2,3");
    assert.sameValue(new constructor([1, 2, 3]).join(null), "1null2null3");
    assert.sameValue(new constructor([1, 2, 3]).join(""), "123");
    assert.sameValue(new constructor([1, 2, 3]).join("+"), "1+2+3");
    assert.sameValue(new constructor([1, 2, 3]).join(.1), "10.120.13");
    assert.sameValue(new constructor([1, 2, 3]).join({toString(){return "foo"}}), "1foo2foo3");
    assert.sameValue(new constructor([1]).join("-"), "1");
    assert.sameValue(new constructor().join(), "");
    assert.sameValue(new constructor().join("*"), "");
    assert.sameValue(new constructor(1).join(), "0");
    assert.sameValue(new constructor(3).join(), "0,0,0");

    assert.throws(TypeError, () => new constructor().join({toString(){throw new TypeError}}));
    assert.throws(TypeError, () => new constructor().join(Symbol()));

    // Called from other globals.
    var join = otherGlobal[constructor.name].prototype.join;
    assert.sameValue(join.call(new constructor([1, 2, 3]), "\t"), "1\t2\t3");

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.join.call(invalidReceiver);
        }, "Assert that join fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).join("\0"), "1\0002\0003");
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    assert.deepEqual(new constructor([null, , NaN]).join(), "0,NaN,NaN");
}

