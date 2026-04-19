// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Case 1: splice() removes an element from the array.
{
    let array = [];
    array.push(0, 1, 2);

    array.constructor = {
        [Symbol.species]: function(n) {
            // Increase the initialized length of the array.
            array.push(3, 4, 5);

            // Make the length property non-writable.
            Object.defineProperty(array, "length", {writable: false});

            return new Array(n);
        }
    }

    assert.throws(TypeError, () => Array.prototype.splice.call(array, 0, 1));

    assert.sameValue(array.length, 6);
    assert.compareArray(array, [1, 2, /* hole */, 3, 4, 5]);
}

// Case 2: splice() adds an element to the array.
{
    let array = [];
    array.push(0, 1, 2);

    array.constructor = {
        [Symbol.species]: function(n) {
            // Increase the initialized length of the array.
            array.push(3, 4, 5);

            // Make the length property non-writable.
            Object.defineProperty(array, "length", {writable: false});

            return new Array(n);
        }
    }

    assert.throws(TypeError, () => Array.prototype.splice.call(array, 0, 0, 123));

    assert.sameValue(array.length, 6);
    assert.compareArray(array, [123, 0, 1, 2, 4, 5]);
}

