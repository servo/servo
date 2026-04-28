// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: >
    array-like objects can be flattened
includes: [compareArray.js]
features: [Array.prototype.flat]
---*/

function getArgumentsObject() {
  return arguments;
}

var a = getArgumentsObject([1], [2]);
var actual = [].flat.call(a);
assert.compareArray(actual, [1, 2], 'The value of actual is expected to be [1, 2]');

a = {
  length: 1,
  0: [1],
};
actual = [].flat.call(a);
assert.compareArray(actual, [1], 'The value of actual is expected to be [1]');

a = {
  length: undefined,
  0: [1],
};
actual = [].flat.call(a);
assert.compareArray(actual, [], 'The value of actual is expected to be []');
