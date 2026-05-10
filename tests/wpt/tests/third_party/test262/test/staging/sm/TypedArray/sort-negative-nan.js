// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
// Test with all floating point typed arrays.
const floatConstructors = anyTypedArrayConstructors.filter(isFloatConstructor);

// Also test with cross-compartment wrapped typed arrays.
{
    const otherGlobal = $262.createRealm().global;
    floatConstructors.push(otherGlobal.Float16Array);
    floatConstructors.push(otherGlobal.Float32Array);
    floatConstructors.push(otherGlobal.Float64Array);
}

function* prod(xs, ys) {
    for (let x of xs) {
        for (let y of ys) {
            yield [x, y];
        }
    }
}

const isLittleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] !== 0;

function seti16(i16, i, v) {
    i16[i] = v;
}

function seti32(i16, i, [hi, lo]) {
    i16[i * 2 + isLittleEndian] = hi;
    i16[i * 2 + !isLittleEndian] = lo;
}

function seti64(i16, i, [hi, hiMid, loMid, lo]) {
    if (isLittleEndian) {
        i16[i * 4] = lo;
        i16[i * 4 + 1] = loMid;
        i16[i * 4 + 2] = hiMid;
        i16[i * 4 + 3] = hi;
    } else {
        i16[i * 4 + 3] = lo;
        i16[i * 4 + 2] = loMid;
        i16[i * 4 + 1] = hiMid;
        i16[i * 4] = hi;
    }
}

const setInt = {
    Float16: seti16,
    Float32: seti32,
    Float64: seti64,
};

const NaNs = {
    Float16: [
        0x7C01|0, // smallest SNaN
        0x7DFF|0, // largest SNaN
        0x7E01|0, // smallest QNaN
        0x7FFF|0, // largest QNaN
        0xFC01|0, // smallest SNaN, sign-bit set
        0xFDFF|0, // largest SNaN, sign-bit set
        0xFE01|0, // smallest QNaN, sign-bit set
        0xFFFF|0, // largest QNaN, sign-bit set
    ],
    Float32: [
        [0x7F80|0, 0x0001|0], // smallest SNaN
        [0x7FBF|0, 0xFFFF|0], // largest SNaN
        [0x7FC0|0, 0x0000|0], // smallest QNaN
        [0x7FFF|0, 0xFFFF|0], // largest QNaN
        [0xFF80|0, 0x0001|0], // smallest SNaN, sign-bit set
        [0xFFBF|0, 0xFFFF|0], // largest SNaN, sign-bit set
        [0xFFC0|0, 0x0000|0], // smallest QNaN, sign-bit set
        [0xFFFF|0, 0xFFFF|0], // largest QNaN, sign-bit set
    ],
    Float64: [
        [0x7FF0|0, 0x0000|0, 0x0000|0, 0x0001|0], // smallest SNaN
        [0x7FF7|0, 0xFFFF|0, 0xFFFF|0, 0xFFFF|0], // largest SNaN
        [0x7FF8|0, 0x0000|0, 0x0000|0, 0x0000|0], // smallest QNaN
        [0x7FFF|0, 0xFFFF|0, 0xFFFF|0, 0xFFFF|0], // largest QNaN
        [0xFFF0|0, 0x0000|0, 0x0000|0, 0x0001|0], // smallest SNaN, sign-bit set
        [0xFFF7|0, 0xFFFF|0, 0xFFFF|0, 0xFFFF|0], // largest SNaN, sign-bit set
        [0xFFF8|0, 0x0000|0, 0x0000|0, 0x0000|0], // smallest QNaN, sign-bit set
        [0xFFFF|0, 0xFFFF|0, 0xFFFF|0, 0xFFFF|0], // largest QNaN, sign-bit set
    ],
};

// %TypedArray%.prototype.sort
const TypedArraySort = Int32Array.prototype.sort;

// Test with small and large typed arrays.
const typedArrayLengths = [16, 4096];

for (const [TA, taLength] of prod(floatConstructors, typedArrayLengths)) {
    let type = TA.name.slice(0, -"Array".length);
    let nansLength = NaNs[type].length;
    let fta = new TA(taLength);
    let i16 = new Int16Array(fta.buffer);

    // Add NaNs in various representations at the start of the typed array.
    for (let i = 0; i < nansLength; ++i) {
        setInt[type](i16, i, NaNs[type][i]);
    }

    // Also add two non-NaN values for testing.
    fta[nansLength] = 123;
    fta[nansLength + 1] = -456;

    // Sort the array and validate sort() sorted all elements correctly.
    TypedArraySort.call(fta);

    // |-456| should be sorted to the start.
    assert.sameValue(fta[0], -456);

    // Followed by a bunch of zeros,
    const zeroOffset = 1;
    const zeroCount = taLength - nansLength - 2;
    for (let i = 0; i < zeroCount; ++i) {
        assert.sameValue(fta[zeroOffset + i], 0, `At offset: ${zeroOffset + i}`);
    }

    // and then |123|.
    assert.sameValue(fta[zeroOffset + zeroCount], 123);

    // And finally the NaNs.
    const nanOffset = zeroCount + 2;
    for (let i = 0; i < nansLength; ++i) {
        // We don't assert a specific NaN value is present, because this is
        // not required by the spec and we don't provide any guarantees NaN
        // values are either unchanged or canonicalized in sort().
        assert.sameValue(fta[nanOffset + i], NaN, `At offset: ${nanOffset + i}`);
    }
}

