// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test ToInteger conversion in %TypedArray%.prototype.set(array|typedArray, offset).

let ta = new Int32Array(4);

// %TypedArray%.prototype.set has two different implementations for typed array
// and non-typed array arguments. Test with both input types.
let emptySources = [[], new Int32Array(0)];
let nonEmptySource = [[0], new Int32Array(1)];
let sources = [...emptySources, ...nonEmptySource];

// Test when ToInteger(offset) is in (-1, 4).
let validOffsets = [
    // Values in [+0, 4).
    0,
    0.1,
    3,
    3.9,

    // Values in (-1, -0].
    -0,
    -0.1,
    -0.9,

    NaN,

    // Also include some non-number values.
    undefined,
    null,
    true,
    "",
    "3",
    "  1\t\n",
    "some string",
    {valueOf() { return 2; }},
];

for (let offset of validOffsets) {
    for (let source of sources) {
        ta.set(source, offset);
    }
}

// Test when ToInteger(offset) isn't in (-1, 4).
let invalidOffsets = [
    // Values exceeding the typed array's length.
    5,
    2147483647,
    2147483648,
    2147483649,
    4294967295,
    4294967296,
    4294967297,
    Infinity,

    // Negative values.
    -1,
    -1.1,
    -2147483647,
    -2147483648,
    -2147483649,
    -4294967295,
    -4294967296,
    -4294967297,
    -Infinity,

    // Also include some non-number values.
    "8",
    "Infinity",
    "  Infinity  ",
    {valueOf() { return 10; }},
];

for (let offset of invalidOffsets) {
    for (let source of sources) {
        assert.throws(RangeError, () => ta.set(source, offset));
    }
}

// Test when ToInteger(offset) is in [4, 5).
for (let source of emptySources) {
    ta.set(source, 4);
    ta.set(source, 4.9);
}
for (let source of nonEmptySource) {
    assert.throws(RangeError, () => ta.set(source, 4));
    assert.throws(RangeError, () => ta.set(source, 4.9));
}

// ToInteger(symbol value) throws a TypeError.
for (let source of sources) {
    assert.throws(TypeError, () => ta.set(source, Symbol()));
}

