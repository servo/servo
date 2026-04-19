// Copyright (C) 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  Stability of Array.prototype.sort for an array with 11 elements.
info: |
  The sort is required to be stable (that is, elements that compare equal
  remain in their original order).
  The array length of 11 was chosen because V8 used an unstable
  QuickSort for arrays with more than 10 elements until v7.0 (September
  2018). https://v8.dev/blog/array-sort
features: [stable-array-sort]
---*/

const array = [
  { name: 'A', rating: 2 },
  { name: 'B', rating: 3 },
  { name: 'C', rating: 2 },
  { name: 'D', rating: 4 },
  { name: 'E', rating: 3 },
  { name: 'F', rating: 3 },
  { name: 'G', rating: 4 },
  { name: 'H', rating: 3 },
  { name: 'I', rating: 2 },
  { name: 'J', rating: 2 },
  { name: 'K', rating: 2 },
];
assert.sameValue(array.length, 11);

// Sort the elements by `rating` in descending order.
// (This updates `array` in place.)
array.sort((a, b) => b.rating - a.rating);

const reduced = array.reduce((acc, element) => acc + element.name, '');
assert.sameValue(reduced, 'DGBEFHACIJK');
