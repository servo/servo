// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, compareArray.js]
description: |
  pending
esid: pending
---*/
// Copies bytes bit-wise if source and target type are the same.
// Only detectable when using floating point typed arrays.
const float32Constructors = anyTypedArrayConstructors.filter(isFloatConstructor)
                                                     .filter(c => c.BYTES_PER_ELEMENT === 4);
const float64Constructors = anyTypedArrayConstructors.filter(isFloatConstructor)
                                                     .filter(c => c.BYTES_PER_ELEMENT === 8);

// Also test with cross-compartment typed arrays.
const otherGlobal = $262.createRealm().global;
float32Constructors.push(otherGlobal.Float32Array);
float64Constructors.push(otherGlobal.Float64Array);

function* p(xs, ys) {
    for (let x of xs) {
        for (let y of ys) {
            yield [x, y];
        }
    }
}

const isLittleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] !== 0;

function geti64(i32, i) {
    return [i32[2 * i + isLittleEndian], i32[2 * i + !isLittleEndian]];
}

function seti64(i32, i, [hi, lo]) {
    i32[i * 2 + isLittleEndian] = hi;
    i32[i * 2 + !isLittleEndian] = lo;
}

const NaNs = {
    Float32: [
        0x7F800001|0, // smallest SNaN
        0x7FBFFFFF|0, // largest SNaN
        0x7FC00000|0, // smallest QNaN
        0x7FFFFFFF|0, // largest QNaN
        0xFF800001|0, // smallest SNaN, sign-bit set
        0xFFBFFFFF|0, // largest SNaN, sign-bit set
        0xFFC00000|0, // smallest QNaN, sign-bit set
        0xFFFFFFFF|0, // largest QNaN, sign-bit set
    ],
    Float64: [
        [0x7FF00000|0, 0x00000001|0], // smallest SNaN
        [0x7FF7FFFF|0, 0xFFFFFFFF|0], // largest SNaN
        [0x7FF80000|0, 0x00000000|0], // smallest QNaN
        [0x7FFFFFFF|0, 0xFFFFFFFF|0], // largest QNaN
        [0xFFF00000|0, 0x00000001|0], // smallest SNaN, sign-bit set
        [0xFFF7FFFF|0, 0xFFFFFFFF|0], // largest SNaN, sign-bit set
        [0xFFF80000|0, 0x00000000|0], // smallest QNaN, sign-bit set
        [0xFFFFFFFF|0, 0xFFFFFFFF|0], // largest QNaN, sign-bit set
    ],
};

const cNaN = {
    Float32: new Int32Array(new Float32Array([NaN]).buffer)[0],
    Float64: geti64(new Int32Array(new Float64Array([NaN]).buffer), 0),
};

// Float32 -> Float32
for (let [sourceConstructor, targetConstructor] of p(float32Constructors, float32Constructors)) {
    let len = NaNs.Float32.length;
    let f32 = new sourceConstructor(len);
    let i32 = new Int32Array(f32.buffer);
    f32.constructor = targetConstructor;

    for (let i = 0; i < len; ++i) {
        i32[i] = NaNs.Float32[i];
    }

    let rf32 = f32.slice(0);
    let ri32 = new Int32Array(rf32.buffer);

    assert.sameValue(rf32.length, len);
    assert.sameValue(ri32.length, len);

    // Same bits.
    for (let i = 0; i < len; ++i) {
        assert.sameValue(ri32[i], NaNs.Float32[i]);
    }
}

// Float64 -> Float64
for (let [sourceConstructor, targetConstructor] of p(float64Constructors, float64Constructors)) {
    let len = NaNs.Float64.length;
    let f64 = new sourceConstructor(len);
    let i32 = new Int32Array(f64.buffer);
    f64.constructor = targetConstructor;

    for (let i = 0; i < len; ++i) {
        seti64(i32, i, NaNs.Float64[i]);
    }

    let rf64 = f64.slice(0);
    let ri32 = new Int32Array(rf64.buffer);

    assert.sameValue(rf64.length, len);
    assert.sameValue(ri32.length, 2 * len);

    // Same bits.
    for (let i = 0; i < len; ++i) {
        assert.compareArray(geti64(ri32, i), NaNs.Float64[i]);
    }
}

