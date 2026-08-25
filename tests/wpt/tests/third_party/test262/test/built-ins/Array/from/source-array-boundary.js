// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Source array with boundary values
esid: sec-array.from
es6id: 22.1.2.1
---*/

var array = [Number.MAX_VALUE, Number.MIN_VALUE, Number.NaN, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY];
var arrayIndex = -1;

function mapFn(value, index) {
  this.arrayIndex++;
  assert.sameValue(value, array[this.arrayIndex], 'The value of value is expected to equal the value of array[this.arrayIndex]');
  assert.sameValue(index, this.arrayIndex, 'The value of index is expected to equal the value of this.arrayIndex');

  return value;
}

var a = Array.from(array, mapFn, this);

assert.sameValue(a.length, array.length, 'The value of a.length is expected to equal the value of array.length');
assert.sameValue(a[0], Number.MAX_VALUE, 'The value of a[0] is expected to equal the value of Number.MAX_VALUE');
assert.sameValue(a[1], Number.MIN_VALUE, 'The value of a[1] is expected to equal the value of Number.MIN_VALUE');
assert.sameValue(a[2], Number.NaN, 'The value of a[2] is expected to equal the value of Number.NaN');
assert.sameValue(a[3], Number.NEGATIVE_INFINITY, 'The value of a[3] is expected to equal the value of Number.NEGATIVE_INFINITY');
assert.sameValue(a[4], Number.POSITIVE_INFINITY, 'The value of a[4] is expected to equal the value of Number.POSITIVE_INFINITY');
