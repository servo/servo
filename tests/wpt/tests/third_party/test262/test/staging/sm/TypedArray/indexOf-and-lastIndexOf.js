// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
// Tests for TypedArray#indexOf.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.indexOf.length, 1);

    // Works with one argument.
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(0), -1);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(1), 0);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(5), 4);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(6), -1);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).indexOf(1), 0);

    if (isFloatConstructor(constructor)) {
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(NaN), -1);
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(0), 1);
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(-0), 1);
    } else {
        // [NaN, 0, -0] will be coerced to [0, 0, 0]
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(NaN), -1);
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(0), 0);
        assert.sameValue(new constructor([NaN, 0, -0]).indexOf(-0), 0);
    }

    // Works with two arguments.
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(1, 1), -1);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(1, -100), 0);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(3, 100), -1);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).indexOf(5, -1), 4);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).indexOf(1, 2), 2);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).indexOf(1, -2), 4);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.indexOf.call(invalidReceiver);
        }, "Assert that indexOf fails if this value is not a TypedArray");
    });

    // test that this.length is never called
    assert.sameValue(Object.defineProperty(new constructor([0, 1, 2, 3, 5]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).indexOf(1), 1);
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    if (constructor.BYTES_PER_ELEMENT === 2) {
        assert.sameValue(new constructor([.1, .2, .3]).indexOf(.2), -1);
        assert.sameValue(new constructor([.1, .2, .3]).indexOf(Math.f16round(.2)), 1);
    } else if (constructor.BYTES_PER_ELEMENT === 4) {
        assert.sameValue(new constructor([.1, .2, .3]).indexOf(.2), -1);
        assert.sameValue(new constructor([.1, .2, .3]).indexOf(Math.fround(.2)), 1);
    } else {
        assert.sameValue(constructor.BYTES_PER_ELEMENT, 8);
        assert.sameValue(new constructor([.1, .2, .3]).indexOf(.2), 1);
    }
}

// Tests for TypedArray#lastIndexOf.
for (var constructor of anyTypedArrayConstructors) {

    assert.sameValue(constructor.prototype.lastIndexOf.length, 1);

    // Works with one arguments.
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(0), -1);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(1), 0);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(5), 4);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(6), -1);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).lastIndexOf(1), 4);

    if (isFloatConstructor(constructor)) {
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(NaN), -1);
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(0), 2);
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(-0), 2);
    } else {
        // [NaN, 0, -0] will be coerced to [0, 0, 0].
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(NaN), -1);
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(0), 2);
        assert.sameValue(new constructor([NaN, 0, -0]).lastIndexOf(-0), 2);
    }

    // Works with two arguments.
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(1, 1), 0);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(1, -100), -1);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(3, 100), 2);
    assert.sameValue(new constructor([1, 2, 3, 4, 5]).lastIndexOf(5, -1), 4);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).lastIndexOf(1, 2), 2);
    assert.sameValue(new constructor([1, 2, 1, 2, 1]).lastIndexOf(1, -2), 2);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.lastIndexOf.call(invalidReceiver);
        }, "Assert that lastIndexOf fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(new constructor([0, 1, 2, 3, 5]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).lastIndexOf(1), 1);

    // Starts search at last index when fromIndex parameter is absent.
    assert.sameValue(new constructor([10, 20, 10]).lastIndexOf(10), 2);

    // Starts search at first index when fromIndex parameter is undefined.
    assert.sameValue(new constructor([10, 20, 10]).lastIndexOf(10, undefined), 0);
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    if (constructor.BYTES_PER_ELEMENT === 2) {
        assert.sameValue(new constructor([.1, .2, .3]).lastIndexOf(.2), -1);
        assert.sameValue(new constructor([.1, .2, .3]).lastIndexOf(Math.f16round(.2)), 1);
    } else if (constructor.BYTES_PER_ELEMENT === 4) {
        assert.sameValue(new constructor([.1, .2, .3]).lastIndexOf(.2), -1);
        assert.sameValue(new constructor([.1, .2, .3]).lastIndexOf(Math.fround(.2)), 1);
    } else {
        assert.sameValue(constructor.BYTES_PER_ELEMENT, 8);
        assert.sameValue(new constructor([.1, .2, .3]).lastIndexOf(.2), 1);
    }
}

