// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [testTypedArray.js, compareArray.js]
description: |
  Sort every possible permutation of some TypedArrays.
esid: pending
features: [TypedArray]
---*/

// Yield every permutation of the elements in some array.
function* Permutations(items) {
  if (items.length === 0) {
    yield [];
  } else {
    for (let i = 0; i < items.length; i++) {
      let tail = items.slice(0);
      let head = tail.splice(i, 1);
      for (let e of Permutations(tail)) {
        yield head.concat(e);
      }
    }
  }
}

const testCases = {
  // Pre-sorted test data, it's important that these arrays remain in ascending order.
  Int8Array: [[-128, 127]],
  Int16Array: [[-32768, -999, 1942, 32767]],
  Int32Array: [[-2147483648, -320000, -244000, 2147483647]],
  Uint8Array: [[255]],
  Uint16Array: [[0, 65535, 65535]],
  Uint32Array: [[0, 987632, 4294967295]],
  Uint8ClampedArray: [[255]],

  // Test the behavior in the default comparator as described in 22.2.3.26.
  // The spec boils down to, -0s come before +0s, and NaNs always come last.
  // Float Arrays are used because all other types convert -0 and NaN to +0.
  Float16Array: [
      [-2147483647, -2147483646.99, -0, 0, 2147483646.99, NaN],
      [1/undefined, NaN, Number.NaN]
  ],
  Float32Array: [
      [-2147483647, -2147483646.99, -0, 0, 2147483646.99, NaN],
      [1/undefined, NaN, Number.NaN]
  ],
  Float64Array: [
      [-2147483646.99, -0, 0, 4147483646.99, NaN],
      [1/undefined, NaN, Number.NaN]
  ],
};

function sortAllPermutations(dataType, data) {
  let reference = new dataType(data);
  for (let permutation of Permutations(data)) {
    let sorted = new dataType(permutation).sort();
    assert.compareArray(sorted, reference);
  }
}

for (let constructor of typedArrayConstructors) {
  for (let data of testCases[constructor.name]) {
    sortAllPermutations(constructor, data);
  }
}

