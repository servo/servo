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

// Tests for TypedArray#every.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.every.length, 1);

    // Basic tests.
    assert.sameValue(new constructor([1, 3, 5]).every(v => v % 2), true);
    assert.sameValue(new constructor([1, 3, 5]).every(v => v > 2), false);
    assert.sameValue(new constructor(10).every(v => v === 0), true);
    assert.sameValue(new constructor().every(v => v > 1), true);

    var arr = new constructor([1, 2, 3, 4, 5]);
    var sum = 0;
    var count = 0;
    assert.sameValue(arr.every((v, k, o) => {
        count++;
        sum += v;
        assert.sameValue(k, v - 1);
        assert.sameValue(o, arr);
        return v < 3;
    }), false);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // Tests for `thisArg` argument.
    function assertThisArg(thisArg, thisValue) {
        // In sloppy mode, `this` could be global object or a wrapper of `thisArg`.
        assert.sameValue(arr.every(function() {
            assert.deepEqual(this, thisValue);
            return true;
        }, thisArg), true);

        // In strict mode, `this` strictly equals `thisArg`.
        assert.sameValue(arr.every(function() {
            "use strict";
            assert.deepEqual(this, thisArg);
            return true;
        }, thisArg), true);

        // Passing `thisArg` has no effect if callback is an arrow function.
        var self = this;
        assert.sameValue(arr.every(() => {
            assert.sameValue(this, self);
            return true;
        }, thisArg), true);
    }
    assertThisArg([1, 2, 3], [1, 2, 3]);
    assertThisArg(Object, Object);
    assertThisArg(1, Object(1));
    assertThisArg("1", Object("1"));
    assertThisArg(false, Object(false));
    assertThisArg(undefined, this);
    assertThisArg(null, this);

    // Throw an exception in the callback.
    var sum = 0;
    var count = 0;
    var thrown = false;
    try {
        arr.every((v, k, o) => {
            count++;
            sum += v;
            assert.sameValue(k, v - 1);
            assert.sameValue(o, arr);
            if (v === 3) {
                throw "every";
            }
            return true
        })
    } catch(e) {
        assert.sameValue(e, "every");
        thrown = true;
    }
    assert.sameValue(thrown, true);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.every();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.every(callback);
        });
    })

    // Callback is a generator.
    arr.every(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var every = otherGlobal[constructor.name].prototype.every;
    var sum = 0;
    assert.sameValue(every.call(new constructor([1, 2, 3]), v => sum += v), true);
    assert.sameValue(sum, 6);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.every.call(invalidReceiver, () => true);
        }, "Assert that every fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).every(() => true), true);
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    assert.sameValue(new constructor([undefined, , NaN]).every(v => Object.is(v, NaN)), true);
}

// Tests for TypedArray#some.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.some.length, 1);

    // Basic tests.
    assert.sameValue(new constructor([1, 2, 3]).some(v => v % 2), true);
    assert.sameValue(new constructor([0, 2, 4]).some(v => v % 2), false);
    assert.sameValue(new constructor([1, 3, 5]).some(v => v > 2), true);
    assert.sameValue(new constructor([1, 3, 5]).some(v => v < 0), false);
    assert.sameValue(new constructor(10).some(v => v !== 0), false);
    assert.sameValue(new constructor().some(v => v > 1), false);

    var arr = new constructor([1, 2, 3, 4, 5]);
    var sum = 0;
    var count = 0;
    assert.sameValue(arr.some((v, k, o) => {
        count++;
        sum += v;
        assert.sameValue(k, v - 1);
        assert.sameValue(o, arr);
        return v > 2;
    }), true);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // Tests for `thisArg` argument.
    function assertThisArg(thisArg, thisValue) {
        // In sloppy mode, `this` could be global object or a wrapper of `thisArg`.
        assert.sameValue(arr.some(function() {
            assert.deepEqual(this, thisValue);
            return false;
        }, thisArg), false);

        // In strict mode, `this` strictly equals `thisArg`.
        assert.sameValue(arr.some(function() {
            "use strict";
            assert.deepEqual(this, thisArg);
            return false;
        }, thisArg), false);

        // Passing `thisArg` has no effect if callback is an arrow function.
        var self = this;
        assert.sameValue(arr.some(() => {
            assert.sameValue(this, self);
            return false;
        }, thisArg), false);
    }
    assertThisArg([1, 2, 3], [1, 2, 3]);
    assertThisArg(Object, Object);
    assertThisArg(1, Object(1));
    assertThisArg("1", Object("1"));
    assertThisArg(false, Object(false));
    assertThisArg(undefined, this);
    assertThisArg(null, this);

    // Throw an exception in the callback.
    var sum = 0;
    var count = 0;
    var thrown = false;
    try {
        arr.some((v, k, o) => {
            count++;
            sum += v;
            assert.sameValue(k, v - 1);
            assert.sameValue(o, arr);
            if (v === 3) {
                throw "some";
            }
            return false
        })
    } catch(e) {
        assert.sameValue(e, "some");
        thrown = true;
    }
    assert.sameValue(thrown, true);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.some();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.some(callback);
        });
    })

    // Callback is a generator.
    arr.some(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var some = otherGlobal[constructor.name].prototype.some;
    var sum = 0;
    assert.sameValue(some.call(new constructor([1, 2, 3]), v => {
        sum += v;
        return false;
    }), false);
    assert.sameValue(sum, 6);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.some.call(invalidReceiver, () => true);
        }, "Assert that some fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).some(() => false), false);
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    assert.sameValue(new constructor([undefined, , NaN]).some(v => v === v), false);
}

