// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Test ToObject in %TypedArray%.prototype.set(array|typedArray, offset).

let ta = new Int32Array(4);

for (let nullOrUndefined of [null, undefined]) {
    // ToObject(array) throws a TypeError when |array| is null or undefined.
    assert.throws(TypeError, () => ta.set(nullOrUndefined));

    // ToInteger(offset) is called before ToObject(array).
    class ExpectedError extends Error {}
    assert.throws(ExpectedError, () => ta.set(nullOrUndefined, {
        valueOf() {
            throw new ExpectedError();
        }
    }));
}

// Ensure ta is still initialized with zeros.
assert.compareArray(ta, [0, 0, 0, 0]);

// %TypedArray%.prototype.set can be called with a string primitive values.
ta.set("");
assert.compareArray(ta, [0, 0, 0, 0]);

ta.set("123");
assert.compareArray(ta, [1, 2, 3, 0]);

// Throws a RangeError if the length is too large.
assert.throws(RangeError, () => ta.set("456789"));
assert.compareArray(ta, [1, 2, 3, 0]);

// When called with other primitive values the typed array contents don't
// change since ToObject(<primitive>).length is zero, i.e. the source object is
// treated the same as an empty array.
for (let value of [true, false, 0, NaN, 123, Infinity, Symbol()]) {
    ta.set(value);
    assert.compareArray(ta, [1, 2, 3, 0]);
}

// Repeat test from above when the primitive wrapper prototype has been changed
// to include "length" and an indexed property.
Number.prototype.length = 4;
Number.prototype[3] = -1;
try {
    ta.set(456);
    assert.compareArray(ta, [0, 0, 0, -1]);
} finally {
    delete Number.prototype.length;
    delete Number.prototype[3];
}

