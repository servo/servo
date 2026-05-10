// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js, compareArray.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

var otherGlobal = $262.createRealm().global;

// Tests for TypedArray#map.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.map.length, 1);

    // Basic tests.
    assert.compareArray(new constructor([1, 3, 5]).map(v => v * 2), new constructor([2,6,10]));
    assert.compareArray(new constructor([-1, 13, 5]).map(v => v - 2), new constructor([-3, 11, 3]));
    assert.compareArray(new constructor(10).map(v => v), new constructor(10));
    assert.compareArray(new constructor().map(v => v + 1), new constructor);
    assert.compareArray(new constructor([1,2,3]).map(v => v), new constructor([1,2,3]));

    var arr = new constructor([1, 2, 3, 4, 5]);
    var sum = 0;
    var count = 0;
    assert.compareArray(arr.map((v, k, o) => {
        count++;
        sum += v;
        assert.sameValue(k, v - 1);
        assert.sameValue(o, arr);
        return v;
    }), arr);
    assert.sameValue(sum, 15);
    assert.sameValue(count, 5);

    // Test that changing elements that have been visited does not affect the result.
    var changeArr = new constructor([1,2,3,4,5]);
    assert.compareArray(arr.map((v,k) => {
        changeArr[k] = v + 1;
        return v;
    }), new constructor([1,2,3,4,5]));

    // Tests for `thisArg` argument.
    function assertThisArg(thisArg, thisValue) {
        // In sloppy mode, `this` could be global object or a wrapper of `thisArg`.
        assert.compareArray(arr.map(function(v) {
            assert.deepEqual(this, thisValue);
            return v;
        }, thisArg), arr);

        // In strict mode, `this` strictly equals `thisArg`.
        assert.compareArray(arr.map(function(v) {
            "use strict";
            assert.deepEqual(this, thisArg);
            return v;
        }, thisArg), arr);

        // Passing `thisArg` has no effect if callback is an arrow function.
        var self = this;
        assert.compareArray(arr.map((v) => {
            assert.sameValue(this, self);
            return v;
        }, thisArg), arr);
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
        arr.map((v, k, o) => {
            count++;
            sum += v;
            assert.sameValue(k, v - 1);
            assert.sameValue(o, arr);
            if (v === 3) {
                throw "map";
            }
            return v;
        })
    } catch(e) {
        assert.sameValue(e, "map");
        thrown = true;
    }
    assert.sameValue(thrown, true);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.map();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.map(callback);
        });
    })

    // Callback is a generator.
    arr.map(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var map = otherGlobal[constructor.name].prototype.map;
    var sum = 0;
    assert.compareArray(map.call(new constructor([1, 2, 3]), v => sum += v), new constructor([1,3,6]));
    assert.sameValue(sum, 6);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.filter.call(invalidReceiver, () => true);
        }, "Assert that map fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.compareArray(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).map((b) => b), new constructor([1,2,3]));
}

// Test For TypedArray#filter.
for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.prototype.filter.length, 1)

    // Basic tests.
    assert.compareArray(new constructor([1,2,3]).filter(x => x == x), new constructor([1,2,3]));
    assert.compareArray(new constructor([1,2,3,4]).filter(x => x % 2 == 0), new constructor([2,4]));
    assert.compareArray(new constructor([1,2,3,4,5]).filter(x => x < 4), new constructor([1,2,3]));
    assert.compareArray(new constructor().filter(x => x * 2 == 4), new constructor());

    var arr = new constructor([1,2,3,4,5]);
    var sum = 0;
    var count = 0;
    assert.compareArray(arr.filter((v, k, o) => {
        count++;
        sum += v;
        assert.sameValue(k, v - 1);
        assert.sameValue(o, arr);
        return (v < 4);
    }), new constructor([1,2,3]));
    assert.sameValue(sum, 15);
    assert.sameValue(count, 5);

    // Test that changing elements that have been visited does not affect the result.
    var changeArr = new constructor([1,2,3,4,5]);
    assert.compareArray(arr.filter((v,k) => {
        changeArr[k] = v + 1;
        return true;
    }), new constructor([1,2,3,4,5]));

    // Tests for `thisArg` argument.
    function assertThisArg(thisArg, thisValue) {
        // In sloppy mode, `this` could be global object or a wrapper of `thisArg`.
        assert.compareArray(arr.filter(function(v) {
            assert.deepEqual(this, thisValue);
            return v;
        }, thisArg), arr);

        // In strict mode, `this` strictly equals `thisArg`.
        assert.compareArray(arr.filter(function(v) {
            "use strict";
            assert.deepEqual(this, thisArg);
            return v;
        }, thisArg), arr);

        // Passing `thisArg` has no effect if callback is an arrow function.
        var self = this;
        assert.compareArray(arr.filter((v) => {
            assert.sameValue(this, self);
            return v;
        }, thisArg), arr);
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
        arr.filter((v, k, o) => {
            count++;
            sum += v;
            assert.sameValue(k, v - 1);
            assert.sameValue(o, arr);
            if (v === 3) {
                throw "filter";
            }
            return v;
        })
    } catch(e) {
        assert.sameValue(e, "filter");
        thrown = true;
    }
    assert.sameValue(thrown, true);
    assert.sameValue(sum, 6);
    assert.sameValue(count, 3);

    // There is no callback or callback is not a function.
    assert.throws(TypeError, () => {
        arr.filter();
    });
    var invalidCallbacks = [undefined, null, 1, false, "", Symbol(), [], {}, /./];
    invalidCallbacks.forEach(callback => {
        assert.throws(TypeError, () => {
            arr.filter(callback);
        });
    })

    // Callback is a generator.
    arr.filter(function*(){
        throw "This line will not be executed";
    });

    // Called from other globals.
    var filter = otherGlobal[constructor.name].prototype.filter;
    var sum = 0;
    assert.compareArray(filter.call(new constructor([1, 2, 3]), v => {sum += v; return true}),
    new constructor([1,2,3]));
    assert.sameValue(sum, 6);

    // Throws if `this` isn't a TypedArray.
    var invalidReceivers = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                            new Proxy(new constructor(), {})];
    invalidReceivers.forEach(invalidReceiver => {
        assert.throws(TypeError, () => {
            constructor.prototype.filter.call(invalidReceiver, () => true);
        }, "Assert that filter fails if this value is not a TypedArray");
    });

    // Test that the length getter is never called.
    assert.compareArray(Object.defineProperty(new constructor([1, 2, 3]), "length", {
        get() {
            throw new Error("length accessor called");
        }
    }).filter((b) => true), new constructor([1,2,3]));
}

// Test that changing Array.prototype[Symbol.iterator] does not affect the
// behaviour of filter. See https://bugzilla.mozilla.org/show_bug.cgi?id=1121936#c18
// for more details.

var arr = new Uint16Array([1,2,3]);

// save
var old = Array.prototype[Symbol.iterator];

Array.prototype[Symbol.iterator] = () => { throw new Error("unreachable"); };
assert.compareArray(arr.filter(v => true), arr);

// restore
Array.prototype[Symbol.iterator] = old;

// Test that defining accessors on Array.prototype doesn't affect the behaviour
// of filter. See https://bugzilla.mozilla.org/show_bug.cgi?id=1121936#c18
// for more details.
Object.defineProperty(Array.prototype, 0, {configurable: true, get: function() { return 1; }, set: function() { this.b = 1; }});
assert.compareArray(new Uint16Array([1,2,3]).filter(v => true), new Uint16Array([1,2,3]));
delete Array.prototype[0];

