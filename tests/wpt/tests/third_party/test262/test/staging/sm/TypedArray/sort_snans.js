// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Ensure that signaling NaN's don't cause problems while sorting

function getNaNArray(length) {
    let a = [];
    for (let i = 0; i < length; i++)
        a.push(NaN);
    return a;
}

// Test every skipNth value in some range n, where start <= n <= end
// and start/stop should be 32-bit integers with bit patterns that
// form Float32 NaNs.
function testFloat32NaNRanges(start, end) {
    let skipN = 10e3;

    // sample the space of possible NaNs to save time
    let sampleSize =  Math.floor((end - start)/ skipN);

    let NaNArray   = new Float32Array(getNaNArray(sampleSize));
    let buffer     = new ArrayBuffer(4 * sampleSize);
    let uintView   = new Uint32Array(buffer);
    let floatView  = new Float32Array(buffer);

    uintView[0] = start;
    for (let i = 1; i < sampleSize; i++) {
        uintView[i] = uintView[0] + (i * skipN);
    }

    floatView.sort();
    assert.deepEqual(floatView, NaNArray);
}

// Test every skipNth value in some range n, where start <= n <= end
// and startHi, startLow and endHi, endLow should be 32-bit integers which,
// when combined (Hi + Low), form Float64 NaNs.
function testFloat64NaNRanges(startHi, startLow, endHi, endLow) {

    // Swap on big endian platforms
    if (new Uint32Array(new Uint8Array([1,2,3,4]).buffer)[0] === 0x01020304) {
	[startHi, startLow] = [startLow, startHi];
	[endHi, endLow] = [endLow, endHi];
    }

    let skipN = 10e6;

    let sampleSizeHi  = Math.floor((endHi - startHi)/skipN);
    let sampleSizeLow = Math.floor((endLow - startLow)/skipN);

    let NaNArray   = new Float64Array(getNaNArray(sampleSizeHi + sampleSizeLow));
    let buffer     = new ArrayBuffer(8 * (sampleSizeHi + sampleSizeLow));
    let uintView   = new Uint32Array(buffer);
    let floatView  = new Float64Array(buffer);

    // Fill in all of the low bits first.
    for (let i = 0; i <= sampleSizeLow; i++) {
        uintView[i * 2] = startLow + (i * skipN);
        uintView[(i * 2) + 1] = startHi;
    }

    // Then the high bits.
    for (let i = sampleSizeLow; i <= sampleSizeLow + sampleSizeHi; i++) {
        uintView[i * 2] = endLow;
        uintView[(i * 2) + 1] = startHi + ((i - sampleSizeLow) * skipN);
    }

    floatView.sort();
    assert.deepEqual(floatView, NaNArray);
}

// Float32 Signaling NaN ranges
testFloat32NaNRanges(0x7F800001, 0x7FBFFFFF);
testFloat32NaNRanges(0xFF800001, 0xFFBFFFFF);

// Float64 Signaling NaN ranges
testFloat64NaNRanges(0x7FF00000, 0x00000001, 0x7FF7FFFF, 0xFFFFFFFF);
testFloat64NaNRanges(0xFFF00000, 0x00000001, 0xFFF7FFFF, 0xFFFFFFFF);

