// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/
const TypedArrayPrototype = Object.getPrototypeOf(Int8Array.prototype);

// %TypedArrayPrototype% has an own "toString" property.
assert.sameValue(TypedArrayPrototype.hasOwnProperty("toString"), true);

// The initial value of %TypedArrayPrototype%.toString is Array.prototype.toString.
assert.sameValue(TypedArrayPrototype.toString, Array.prototype.toString);

// The concrete TypedArray prototypes do not have an own "toString" property.
assert.sameValue(anyTypedArrayConstructors.every(c => !c.hasOwnProperty("toString")), true);

verifyProperty(TypedArrayPrototype, "toString", {
    value: TypedArrayPrototype.toString,
    writable: true,
    enumerable: false,
    configurable: true,
}, {
    restore: true
});

for (let constructor of anyTypedArrayConstructors) {
    assert.sameValue(new constructor([]).toString(), "");
    assert.sameValue(new constructor([1]).toString(), "1");
    assert.sameValue(new constructor([1, 2]).toString(), "1,2");
}

const testCases = {
    [Int8Array.name]: {
        array: [-1, 2, -3, 4, NaN],
        expected: "-1,2,-3,4,0",
    },
    [Int16Array.name]: {
        array: [-1, 2, -3, 4, NaN],
        expected: "-1,2,-3,4,0",
    },
    [Int32Array.name]: {
        array: [-1, 2, -3, 4, NaN],
        expected: "-1,2,-3,4,0",
    },
    [Uint8Array.name]: {
        array: [255, 2, 3, 4, NaN],
        expected: "255,2,3,4,0",
    },
    [Uint16Array.name]: {
        array: [-1, 2, 3, 4, NaN],
        expected: "65535,2,3,4,0",
    },
    [Uint32Array.name]: {
        array: [-1, 2, 3, 4, NaN],
        expected: "4294967295,2,3,4,0",
    },
    [Uint8ClampedArray.name]: {
        array: [255, 256, 2, 3, 4, NaN],
        expected: "255,255,2,3,4,0",
    },
    [Float16Array.name]: {
        array: [-0, 0, 0.5, -0.5, NaN, Infinity, -Infinity],
        expected: "0,0,0.5,-0.5,NaN,Infinity,-Infinity",
    },
    [Float32Array.name]: {
        array: [-0, 0, 0.5, -0.5, NaN, Infinity, -Infinity],
        expected: "0,0,0.5,-0.5,NaN,Infinity,-Infinity",
    },
    [Float64Array.name]: {
        array: [-0, 0, 0.5, -0.5, NaN, Infinity, -Infinity],
        expected: "0,0,0.5,-0.5,NaN,Infinity,-Infinity",
    },
};
for (let constructor of anyTypedArrayConstructors) {
    let {array, expected} = testCases[constructor.name];
    assert.sameValue(new constructor(array).toString(), expected);
}
