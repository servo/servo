// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js, sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // %TypedArray%.from throws if the argument is undefined or null.
    assert.throws(TypeError, () => constructor.from());
    assert.throws(TypeError, () => constructor.from(undefined));
    assert.throws(TypeError, () => constructor.from(null));

    // Unlike Array.from, %TypedArray%.from doesn't get or set the length property.
    function ObjectWithThrowingLengthGetterSetter(...rest) {
        var ta = new constructor(...rest);
        Object.defineProperty(ta, "length", {
            configurable: true,
            get() { throw new RangeError("getter!"); },
            set() { throw new RangeError("setter!"); }
        });
        return ta;
    }
    ObjectWithThrowingLengthGetterSetter.from = constructor.from;
    assert.sameValue(ObjectWithThrowingLengthGetterSetter.from([123])[0], 123);

    // %TypedArray%.from throws if mapfn is neither callable nor undefined.
    assert.throws(TypeError, () => constructor.from([3, 4, 5], {}));
    assert.throws(TypeError, () => constructor.from([3, 4, 5], "also not a function"));
    assert.throws(TypeError, () => constructor.from([3, 4, 5], null));

    // Even if the function would not have been called.
    assert.throws(TypeError, () => constructor.from([], JSON));

    // If mapfn is not undefined and not callable, the error happens before anything else.
    // Before calling the constructor, before touching the arrayLike.
    var log = "";
    var obj;
    function C(...rest) {
        log += "C";
        obj = new constructor(...rest);
        return obj;
    }
    var p = new Proxy({}, {
        has: function () { log += "1"; },
        get: function () { log += "2"; },
        getOwnPropertyDescriptor: function () { log += "3"; }
    });
    assert.throws(TypeError, () => constructor.from.call(C, p, {}));
    assert.sameValue(log, "");

    // If mapfn throws, the new object has already been created.
    var arrayish = {
        get length() { log += "l"; return 1; },
        get 0() { log += "0"; return "q"; }
    };
    log = "";
    var exc = {surprise: "ponies"};
    assertThrowsValue(() => constructor.from.call(C, arrayish, () => { throw exc; }), exc);
    assert.sameValue(log, "lC0");
    assert.sameValue(obj instanceof constructor, true);

    // It's a TypeError if the @@iterator property is a primitive (except null and undefined).
    for (var primitive of ["foo", 17, Symbol(), true]) {
        assert.throws(TypeError, () => constructor.from({[Symbol.iterator] : primitive}));
    }
    assert.deepEqual(constructor.from({[Symbol.iterator]: null}), new constructor());
    assert.deepEqual(constructor.from({[Symbol.iterator]: undefined}), new constructor());

    // It's a TypeError if the iterator's .next() method returns a primitive.
    for (var primitive of [undefined, null, "foo", 17, Symbol(), true]) {
        assert.throws(TypeError,
            () => constructor.from({
                [Symbol.iterator]() {
                    return {next() { return primitive; }};
                }
            }));
    }
}

