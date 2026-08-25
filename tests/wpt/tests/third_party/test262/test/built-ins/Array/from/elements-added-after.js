// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Elements added after the call to from
esid: sec-array.from
---*/

var arrayIndex = -1;
var originalLength = 7;
var obj = {
  length: originalLength,
  0: 2,
  1: 4,
  2: 8,
  3: 16,
  4: 32,
  5: 64,
  6: 128
};
var array = [2, 4, 8, 16, 32, 64, 128];

function mapFn(value, index) {
  arrayIndex++;
  assert.sameValue(value, obj[arrayIndex], 'The value of value is expected to equal the value of obj[arrayIndex]');
  assert.sameValue(index, arrayIndex, 'The value of index is expected to equal the value of arrayIndex');
  obj[originalLength + arrayIndex] = 2 * arrayIndex + 1;

  return obj[arrayIndex];
}


var a = Array.from(obj, mapFn);
assert.sameValue(a.length, array.length, 'The value of a.length is expected to equal the value of array.length');

for (var j = 0; j < a.length; j++) {
  assert.sameValue(a[j], array[j], 'The value of a[j] is expected to equal the value of array[j]');
}
