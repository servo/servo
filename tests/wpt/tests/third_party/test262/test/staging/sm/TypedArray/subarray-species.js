// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
function test(constructor, constructor2, from=[1, 2, 3, 4, 5], to=[3, 4], begin=2, end=4) {
    var modifiedConstructor = new constructor(from);
    modifiedConstructor.constructor = constructor2;
    assert.deepEqual(modifiedConstructor.subarray(begin, end), new constructor2(to));
    var modifiedSpecies = new constructor(from);
    modifiedSpecies.constructor = { [Symbol.species]: constructor2 };
    assert.deepEqual(modifiedSpecies.subarray(begin, end), new constructor2(to));
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

test(Int8Array, Uint8Array, [-1, -2, -3, -4, -5], [0xFD, 0xFC]);
test(Int8Array, Uint8ClampedArray, [-1, -2, -3, -4, -5], [0xFD, 0xFC]);

test(Uint8Array, Int8Array, [0xFF, 0xFE, 0xFD, 0xFC, 0xFB], [-3, -4]);
test(Uint8ClampedArray, Int8Array, [0xFF, 0xFE, 0xFD, 0xFC, 0xFB], [-3, -4]);

test(Int16Array, Uint16Array, [-1, -2, -3, -4, -5], [0xFFFD, 0xFFFC]);
test(Uint16Array, Int16Array, [0xFFFF, 0xFFFE, 0xFFFD, 0xFFFC, 0xFFFB], [-3, -4]);

test(Int32Array, Uint32Array, [-1, -2, -3, -4, -5], [0xFFFFFFFD, 0xFFFFFFFC]);
test(Uint32Array, Int32Array, [0xFFFFFFFF, 0xFFFFFFFE, 0xFFFFFFFD, 0xFFFFFFFC, 0xFFFFFFFB], [-3, -4]);

// different size

// To avoid handling endian, use ArrayBuffer as an argument.
var a = new Int8Array([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                       0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01,
                       0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                       0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x0F]);

test(Uint8Array, Uint16Array, a.buffer, a.slice(2, 6).buffer);
test(Uint16Array, Uint8Array, a.buffer, a.slice(4, 6).buffer);

test(Uint8Array, Uint32Array, a.buffer, a.slice(4, 12).buffer, 4, 6);
test(Uint32Array, Uint8Array, a.buffer, a.slice(8, 10).buffer);

test(Uint16Array, Uint32Array, a.buffer, a.slice(4, 12).buffer);
test(Uint32Array, Uint16Array, a.buffer, a.slice(8, 12).buffer);

test(Float32Array, Float64Array, a.buffer, a.slice(8, 24).buffer);
test(Float64Array, Float32Array, a.buffer, a.slice(16, 24).buffer);

