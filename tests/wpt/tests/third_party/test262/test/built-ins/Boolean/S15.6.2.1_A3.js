// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Value]] property of the newly constructed object
    is set to ToBoolean(value)
esid: sec-boolean-constructor
description: Checking value of the newly created object
---*/

// CHECK#1
var x1 = new Boolean(1);
assert.sameValue(x1.valueOf(), true, 'x1.valueOf() must return true');

var x2 = new Boolean();
assert.sameValue(x2.valueOf(), false, 'x2.valueOf() must return false');

var x2 = new Boolean(0);
assert.sameValue(x2.valueOf(), false, 'x2.valueOf() must return false');

var x2 = new Boolean(new Object());
assert.sameValue(x2.valueOf(), true, 'x2.valueOf() must return true');
