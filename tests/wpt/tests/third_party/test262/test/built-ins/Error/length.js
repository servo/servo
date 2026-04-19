// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property value is 1
es5id: 15.11.3_A2_T1
description: Checking length property
---*/

var err1 = Error("err");
assert.sameValue(err1.constructor.length, 1, 'The value of err1.constructor.length is 1');
assert.sameValue(Error.constructor.length, 1, 'The value of Error.constructor.length is 1');
