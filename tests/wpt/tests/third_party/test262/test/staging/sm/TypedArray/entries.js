// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/

var otherGlobal = $262.createRealm().global;

for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.entries.length, 0);
    assert.sameValue(constructor.prototype.entries.name, "entries");

    assert.deepEqual([...new constructor(0).entries()], []);
    assert.deepEqual([...new constructor(1).entries()], [[0, 0]]);
    assert.deepEqual([...new constructor(2).entries()], [[0, 0], [1, 0]]);
    assert.deepEqual([...new constructor([15]).entries()], [[0, 15]]);

    var arr = new constructor([1, 2, 3]);
    var iterator = arr.entries();
    assert.deepEqual(iterator.next(), {value: [0, 1], done: false});
    assert.deepEqual(iterator.next(), {value: [1, 2], done: false});
    assert.deepEqual(iterator.next(), {value: [2, 3], done: false});
    assert.deepEqual(iterator.next(), {value: undefined, done: true});

    // Called from other globals.
    var entries = otherGlobal[constructor.name].prototype.entries;
    assert.deepEqual([...entries.call(new constructor(2))],
                 [new otherGlobal.Array(0, 0), new otherGlobal.Array(1, 0)]);
    arr = new (otherGlobal[constructor.name])(2);
    assert.sameValue([...constructor.prototype.entries.call(arr)].toString(), "0,0,1,0");

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.entries.call(invalidReceiver);
        }, "Assert that entries fails if this value is not a TypedArray");
    });
}

