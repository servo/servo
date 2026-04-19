// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
function test(constructor, constructor2, from=[1, 2, 3, 4, 5], to=[2, 4, 6, 8, 10]) {
    var modifiedConstructor = new constructor(from);
    modifiedConstructor.constructor = constructor2;
    assert.deepEqual(modifiedConstructor.map(x => x * 2), new constructor2(to));
    var modifiedSpecies = new constructor(from);
    modifiedSpecies.constructor = { [Symbol.species]: constructor2 };
    assert.deepEqual(modifiedSpecies.map(x => x * 2), new constructor2(to));
}

// same size, same sign

test(Int8Array, Uint8Array);
test(Int8Array, Uint8ClampedArray);

test(Uint8Array, Int8Array);
test(Uint8Array, Uint8ClampedArray);

test(Uint8ClampedArray, Int8Array);
test(Uint8ClampedArray, Uint8Array);

test(Int16Array, Uint16Array);
test(Uint16Array, Int16Array);

test(Int32Array, Uint32Array);
test(Uint32Array, Int32Array);

// same size, different sign

test(Int8Array, Uint8Array, [-1, -2, -3, -4, -5], [0xFE, 0xFC, 0xFA, 0xF8, 0xF6]);
test(Int8Array, Uint8ClampedArray, [-1, -2, -3, -4, -5], [0, 0, 0, 0, 0]);

test(Uint8Array, Int8Array, [0xFF, 0xFE, 0xFD, 0xFC, 0xFB], [-2, -4, -6, -8, -10]);
test(Uint8ClampedArray, Int8Array, [0xFF, 0xFE, 0xFD, 0xFC, 0xFB], [-2, -4, -6, -8, -10]);

test(Int16Array, Uint16Array, [-1, -2, -3, -4, -5], [0xFFFE, 0xFFFC, 0xFFFA, 0xFFF8, 0xFFF6]);
test(Uint16Array, Int16Array, [0xFFFF, 0xFFFE, 0xFFFD, 0xFFFC, 0xFFFB], [-2, -4, -6, -8, -10]);

test(Int32Array, Uint32Array, [-1, -2, -3, -4, -5], [0xFFFFFFFE, 0xFFFFFFFC, 0xFFFFFFFA, 0xFFFFFFF8, 0xFFFFFFF6]);
test(Uint32Array, Int32Array, [0xFFFFFFFF, 0xFFFFFFFE, 0xFFFFFFFD, 0xFFFFFFFC, 0xFFFFFFFB], [-2, -4, -6, -8, -10]);

// different size

test(Uint8Array, Uint16Array);
test(Uint16Array, Uint8Array);

test(Uint8Array, Uint32Array);
test(Uint32Array, Uint8Array);

test(Uint16Array, Uint32Array);
test(Uint32Array, Uint16Array);

test(Float32Array, Float64Array);
test(Float64Array, Float32Array);

