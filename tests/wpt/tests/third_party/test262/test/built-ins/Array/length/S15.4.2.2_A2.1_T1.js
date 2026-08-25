// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
info: |
    If the argument len is a Number and ToUint32(len) is equal to len,
    then the length property of the newly constructed object is set to ToUint32(len)
es5id: 15.4.2.2_A2.1_T1
description: Array constructor is given one argument
---*/

var x = new Array(0);
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');

var x = new Array(1);
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

var x = new Array(4294967295);
assert.sameValue(x.length, 4294967295, 'The value of x.length is expected to be 4294967295');
