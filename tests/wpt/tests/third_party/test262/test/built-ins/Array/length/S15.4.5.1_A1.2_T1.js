// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-exotic-objects-defineownproperty-p-desc
info: |
    For every integer k that is less than the value of
    the length property of A but not less than ToUint32(length),
    if A itself has a property (not an inherited property) named ToString(k),
    then delete that property
es5id: 15.4.5.1_A1.2_T1
description: Change length of array
---*/

var x = [0, , 2, , 4];
x.length = 4;
assert.sameValue(x[4], undefined, 'The value of x[4] is expected to equal undefined');

x.length = 3;
assert.sameValue(x[3], undefined, 'The value of x[3] is expected to equal undefined');
assert.sameValue(x[2], 2, 'The value of x[2] is expected to be 2');
