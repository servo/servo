// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // Note %TypedArray%.from(iterable) calls 'this' with an argument whose value is
    // `[...iterable].length`, while Array.from(iterable) doesn't pass any argument.
    constructor.from.call(function(len){
        assert.sameValue(len, 3);
        return new constructor(len);
    }, Array(3));

    // If the 'this' value passed to %TypedArray.from is not a constructor,
    // then an exception is thrown, while Array.from will use Array as it's constructor.
    var arr = [3, 4, 5];
    var nonconstructors = [
        {}, Math, Object.getPrototypeOf, undefined, 17,
        () => ({})  // arrow functions are not constructors
    ];
    for (var v of nonconstructors) {
        assert.throws(TypeError, () => {
            constructor.from.call(v, arr);
        });
    }

    // %TypedArray%.from does not get confused if global constructors for typed arrays
    // are replaced with another constructor.
    function NotArray(...rest) {
        return new constructor(...rest);
    }
    var RealArray = constructor;
    NotArray.from = constructor.from;
    this[constructor.name] = NotArray;
    assert.sameValue(RealArray.from([1]) instanceof RealArray, true);
    assert.sameValue(NotArray.from([1]) instanceof RealArray, true);
    this[constructor.name] = RealArray;
}

