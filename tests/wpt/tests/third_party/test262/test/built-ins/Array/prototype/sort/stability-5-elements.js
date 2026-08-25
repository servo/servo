// Copyright (C) 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  Stability of Array.prototype.sort for an array with 5 elements.
info: |
  The sort is required to be stable (that is, elements that compare equal
  remain in their original order).
features: [stable-array-sort]
---*/

const array = [
  { name: 'A', rating: 2 },
  { name: 'B', rating: 3 },
  { name: 'C', rating: 2 },
  { name: 'D', rating: 3 },
  { name: 'E', rating: 3 },
];
assert.sameValue(array.length, 5);

// Sort the elements by `rating` in descending order.
// (This updates `array` in place.)
array.sort((a, b) => b.rating - a.rating);

const reduced = array.reduce((acc, element) => acc + element.name, '');
assert.sameValue(reduced, 'BDEAC');
