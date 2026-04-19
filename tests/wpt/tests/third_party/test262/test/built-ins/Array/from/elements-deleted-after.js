// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
    Elements deleted after the call started and before visited are not
    visited
esid: sec-array.from
---*/

var originalArray = [0, 1, -2, 4, -8, 16];
var array = [0, 1, -2, 4, -8, 16];
var a = [];
var arrayIndex = -1;

function mapFn(value, index) {
  this.arrayIndex++;
  assert.sameValue(value, array[this.arrayIndex], 'The value of value is expected to equal the value of array[this.arrayIndex]');
  assert.sameValue(index, this.arrayIndex, 'The value of index is expected to equal the value of this.arrayIndex');

  array.splice(array.length - 1, 1);
  return 127;
}


a = Array.from(array, mapFn, this);

assert.sameValue(a.length, originalArray.length / 2, 'The value of a.length is expected to be originalArray.length / 2');

for (var j = 0; j < originalArray.length / 2; j++) {
  assert.sameValue(a[j], 127, 'The value of a[j] is expected to be 127');
}
