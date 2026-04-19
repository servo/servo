// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
// Counting sort instead of radix sort is only used for large two-byte typed arrays.
const TwoByteTypedArrays = anyTypedArrayConstructors.filter(ta => ta.BYTES_PER_ELEMENT === 2);

// The typed array must be large enough to prefer counting sort instead of
// radix sort.
//
// The exact condition is: 65536 * (sizeof(size_t) / sizeof(T)) - constant.
//
// Where `sizeof(T) == 2` and `sizeof(size_t)` is either 4 (32-bit) or 8 (64-bit).
// Use `sizeof(size_t) == 8` which works on both 32- and 64-bit targets.
const LARGE_LENGTH = 65536 * 4;

for (let TA of TwoByteTypedArrays) {
  // Create a large enough typed array.
  let ta = new TA(LARGE_LENGTH);

  // Fill the typed array with the smallest value depending on its type.
  let smallest = isFloatConstructor(TA) ? -Infinity : 
                 isUnsignedConstructor(TA) ? 0 : -32768;
  ta.fill(smallest);

  // Write the sample values into the middle of the typed array, so we can
  // easily determine if the sort operation worked correctly.
  const offset = 100_000;

  // Write all possible values into the array. Use the unsigned representation
  // to ensure all possible byte patterns are used.
  let unsigned = new Uint16Array(ta.buffer);
  for (let i = 0; i < 65536; ++i) {
    unsigned[offset + i] = i;
  }

  // Sort the typed array.
  ta.sort();

  // The smallest value is moved to the start. These are exactly 1 + 65536 * 3
  // elements.
  for (let i = 0; i <= 65536 * 3; ++i) {
    assert.sameValue(ta[i], smallest);
  }

  // The last 65536 elements are sorted in order.
  for (let i = 65536 * 3; i < ta.length - 1; ++i) {
    // Not `ta[i] < ta[i + 1]` in order to handle NaN values correctly.
    assert.sameValue(!(ta[i] > ta[i + 1]), true);
  }
}

