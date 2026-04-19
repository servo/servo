// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
    Behavior when array is depth more than 1
includes: [compareArray.js]
features: [Array.prototype.flatMap]
---*/

assert.compareArray([1, 2].flatMap(function(e) {
  return [e, e * 2];
}), [1, 2, 2, 4], '[1, 2].flatMap(function(e) {return [e, e * 2];}) must return [1, 2, 2, 4]');

var result = [1, 2, 3].flatMap(function(ele) {
  return [
    [ele * 2]
  ];
});
assert.sameValue(result.length, 3, 'The value of result.length is expected to be 3');
assert.compareArray(result[0], [2], 'The value of result[0] is expected to be [2]');
assert.compareArray(result[1], [4], 'The value of result[1] is expected to be [4]');
assert.compareArray(result[2], [6], 'The value of result[2] is expected to be [6]');
