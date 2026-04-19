// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
info: |
    If the argument len is not a Number, then the length property of
    the newly constructed object is set to 1 and the 0 property of
    the newly constructed object is set to len
es5id: 15.4.2.2_A2.3_T5
description: Checking for Number object
---*/

var obj = new Number(-1);
var x = new Array(obj);

assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');
assert.sameValue(x[0], obj, 'The value of x[0] is expected to equal the value of obj');

var obj = new Number(4294967296);
var x = new Array(obj);

assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');
assert.sameValue(x[0], obj, 'The value of x[0] is expected to equal the value of obj');

var obj = new Number(4294967297);
var x = new Array(obj);

assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');
assert.sameValue(x[0], obj, 'The value of x[0] is expected to equal the value of obj');
