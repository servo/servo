// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the Math.max method is 2
es5id: 15.8.2.11_A4
description: Checking if Math.max.length property is defined and equals to 2
---*/
assert.sameValue(typeof Math.max, "function", 'The value of `typeof Math.max` is expected to be "function"');
assert.notSameValue(typeof Math.max.length, "undefined", 'The value of typeof Math.max.length is not "undefined"');
assert.sameValue(Math.max.length, 2, 'The value of Math.max.length is expected to be 2');
