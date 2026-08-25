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

// Tests for TypedArray#reduce.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.reduce.length, 1);

    // Basic tests.
    var arr = new constructor([1, 2, 3, 4, 5]);

    assert.sameValue(arr.reduce((previous, current) => previous + current), 15);
    assert.sameValue(arr.reduce((previous, current) => current - previous), 3);

    var count = 0;
    var sum = 0;
    assert.sameValue(arr.reduce((previous, current, index, array) => {
        count++;
        sum += current;
        assert.sameValue(current - 1, index);
        assert.sameValue(current, arr[index]);
        assert.sameValue(array, arr);
        return previous * current;
    }), 120);
    assert.sameValue(count, 4);
    assert.sameValue(sum, 14);

    // Tests for `initialValue` argument.
    assert.sameValue(arr.reduce((previous, current) => previous + current, -15), 0);
    assert.sameValue(arr.reduce((previous, current) => previous + current, ""), "12345");
    assert.deepEqual(arr.reduce((previous, current) => previous.concat(current), []), [1, 2, 3, 4, 5]);

    // Tests for `this` value.
    var global = this;
    arr.reduce(function(){
        assert.sameValue(this, global);
    });
    arr.reduce(function(){
        "use strict";
        assert.sameValue(this, undefined);
    });
    arr.reduce(() => assert.sameValue(this, global));

    // Throw an exception in the callback.
    var count = 0;
    var sum = 0;
    assert.throws(TypeError, () => {
        arr.reduce((previous, current, index, array) => {
            count++;
            sum += current;
            if (index === 3) {
                throw TypeError("reduce");
            }
        })
    });
    assert.sameValue(count, 3);
    assert.sameValue(sum, 9);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.reduce();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.reduce(callback);
        });
    })

    // Callback is a generator.
    arr.reduce(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var reduce = otherGlobal[constructor.name].prototype.reduce;
    assert.sameValue(reduce.call(arr, (previous, current) => Math.min(previous, current)), 1);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(3), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.reduce.call(invalidReceiver, () => {});
        }, "Assert that reduce fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(arr, "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).reduce((previous, current) => Math.max(previous, current)), 5);
}

// Tests for TypedArray#reduceRight.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.reduceRight.length, 1);

    // Basic tests.
    var arr = new constructor([1, 2, 3, 4, 5]);

    assert.sameValue(arr.reduceRight((previous, current) => previous + current), 15);
    assert.sameValue(arr.reduceRight((previous, current) => current - previous), 3);

    var count = 0;
    var sum = 0;
    assert.sameValue(arr.reduceRight((previous, current, index, array) => {
        count++;
        sum += current;
        assert.sameValue(current - 1, index);
        assert.sameValue(current, arr[index]);
        assert.sameValue(array, arr);
        return previous * current;
    }), 120);
    assert.sameValue(count, 4);
    assert.sameValue(sum, 10);

    // Tests for `initialValue` argument.
    assert.sameValue(arr.reduceRight((previous, current) => previous + current, -15), 0);
    assert.sameValue(arr.reduceRight((previous, current) => previous + current, ""), "54321");
    assert.deepEqual(arr.reduceRight((previous, current) => previous.concat(current), []), [5, 4, 3, 2, 1]);

    // Tests for `this` value.
    var global = this;
    arr.reduceRight(function(){
        assert.sameValue(this, global);
    });
    arr.reduceRight(function(){
        "use strict";
        assert.sameValue(this, undefined);
    });
    arr.reduceRight(() => assert.sameValue(this, global));

    // Throw an exception in the callback.
    var count = 0;
    var sum = 0;
    assert.throws(TypeError, () => {
        arr.reduceRight((previous, current, index, array) => {
            count++;
            sum += current;
            if (index === 1) {
                throw TypeError("reduceRight");
            }
        })
    });
    assert.sameValue(count, 3);
    assert.sameValue(sum, 9);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.reduceRight();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.reduceRight(callback);
        });
    })

    // Callback is a generator.
    arr.reduceRight(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var reduceRight = otherGlobal[constructor.name].prototype.reduceRight;
    assert.sameValue(reduceRight.call(arr, (previous, current) => Math.min(previous, current)), 1);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(3), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.reduceRight.call(invalidReceiver, () => {});
        }, "Assert that reduceRight fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.sameValue(Object.defineProperty(arr, "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).reduceRight((previous, current) => Math.max(previous, current)), 5);
}

