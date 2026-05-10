// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
info: |
    If the argument len is not a Number, then the length property of
    the newly constructed object is set to 1 and the 0 property of
    the newly constructed object is set to len
es5id: 15.4.2.2_A2.3_T3
description: Checking for boolean primitive and Boolean object
---*/

var x = new Array("1");

assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');
assert.sameValue(x[0], "1", 'The value of x[0] is expected to be "1"');

var obj = new String("0");
var x = new Array(obj);

assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');
assert.sameValue(x[0], obj, 'The value of x[0] is expected to equal the value of obj');
