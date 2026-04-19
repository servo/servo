// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    For every integer k that is less than the value of
    the length property of A but not less than ToUint32(length),
    if A itself has a property (not an inherited property) named ToString(k),
    then delete that property
es5id: 15.4.5.1_A1.2_T2
description: Checking an inherited property
---*/

Array.prototype[2] = -1;
var x = [0, 1, 2];
assert.sameValue(x[2], 2, 'The value of x[2] is expected to be 2');

x.length = 2;
assert.sameValue(x[2], -1, 'The value of x[2] is expected to be -1');
