// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Value]] property of the newly constructed object
    is set to ToNumber(value) if value was supplied, else to +0
es5id: 15.7.2.1_A3
description: Checking value of the newly created object
---*/

var x1 = new Number(1);
assert.sameValue(x1.valueOf(), 1, 'x1.valueOf() must return 1');

var x2 = new Number();
assert.sameValue(x2.valueOf(), 0, 'x2.valueOf() must return 0');
