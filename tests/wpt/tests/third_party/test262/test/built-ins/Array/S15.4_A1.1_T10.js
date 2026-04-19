// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T10
description: Array index is power of two
---*/

var x = [];
var k = 1;
for (var i = 0; i < 32; i++) {
  k = k * 2;
  x[k - 2] = k;
}

k = 1;
for (i = 0; i < 32; i++) {
  k = k * 2;
  assert.sameValue(x[k - 2], k, 'The value of x[k - 2] is expected to equal the value of k');
}
