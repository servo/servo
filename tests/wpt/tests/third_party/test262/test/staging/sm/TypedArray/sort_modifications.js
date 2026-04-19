// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
const TAConstructors = [
  Int8Array,
  Uint8Array,
  Int16Array,
  Uint16Array,
  Int32Array,
  Uint32Array,
  Uint8ClampedArray,
  Float32Array,
  Float64Array,
  BigInt64Array,
  BigUint64Array,
].concat(this.Float16Array ?? []);

// Use different size classes to catch any implementation-specific
// optimisations.
const sizes = [
  4, 8, 64, 128, 1024
];

function ToNumeric(TA) {
  if (TA === BigInt64Array || TA === BigUint64Array) {
    return BigInt;
  }
  return Number;
}

function ascending(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function descending(a, b) {
  return -ascending(a, b);
}

for (let TA of TAConstructors) {
  let toNumeric = ToNumeric(TA);
  for (let size of sizes) {
    let sorted = new TA(size);

    // Fill with |1..size| and then sort to account for wrap-arounds.
    for (let i = 0; i < size; ++i) {
      sorted[i] = toNumeric(i + 1);
    }
    sorted.sort();

    // Create a copy in descending order.
    let ta = new TA(sorted);
    ta.sort(descending);

    // Sort the copy in ascending order and on the first call reset all
    // elements to zero.
    let called = false;
    ta.sort(function(a, b) {
      if (!called) {
        called = true;
        ta.fill(toNumeric(0));
      }
      return ascending(a, b);
    });

    // Ensure the comparator function was called.
    assert.sameValue(called, true);

    // All elements should be sorted correctly. No elements should be zero.
    for (let i = 0; i < size; ++i) {
      assert.sameValue(ta[i], sorted[i], `${TA.name} at index ${i} for size ${size}`);
    }
  }
}

