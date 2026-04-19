// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Elements are updated after the call to from
esid: sec-array.from
---*/

var array = [127, 4, 8, 16, 32, 64, 128];
var arrayIndex = -1;

function mapFn(value, index) {
  arrayIndex++;
  if (index + 1 < array.length) {
    array[index + 1] = 127;
  }
  assert.sameValue(value, 127, 'The value of value is expected to be 127');
  assert.sameValue(index, arrayIndex, 'The value of index is expected to equal the value of arrayIndex');

  return value;
}

var a = Array.from(array, mapFn);
assert.sameValue(a.length, array.length, 'The value of a.length is expected to equal the value of array.length');
for (var j = 0; j < a.length; j++) {
  assert.sameValue(a[j], 127, 'The value of a[j] is expected to be 127');
}
