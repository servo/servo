// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Sort every possible permutation of some arrays.
esid: pending
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

function sortAllPermutations(data, comparefn) {
  for (let permutation of Permutations(data)) {
    permutation.sort(comparefn);
    assert.compareArray(permutation, data);
  }
}

let lex  = [2112, "bob", "is", "my", "name"];
let nans = [1/undefined, NaN, Number.NaN]

let num1  = [-11, 0, 0, 100, 101];
let num2  = [-11, 100, 201234.23, undefined, undefined];

sortAllPermutations(lex);
sortAllPermutations(nans);

sortAllPermutations(nans, (x, y) => x - y);
// Multiplication kills comparator optimization.
sortAllPermutations(nans, (x, y) => (1*x - 1*y));

sortAllPermutations(num1, (x, y) => x - y);
sortAllPermutations(num1, (x, y) => (1*x - 1*y));

sortAllPermutations(num2, (x, y) => x - y);
sortAllPermutations(num2, (x, y) => (1*x - 1*y));

