// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the Math.min method is 2
es5id: 15.8.2.12_A4
description: Checking if Math.min.length property is defined and equals to 2
---*/
assert.sameValue(typeof Math.min, "function", 'The value of `typeof Math.min` is expected to be "function"');
assert.notSameValue(typeof Math.min.length, "undefined", 'The value of typeof Math.min.length is not "undefined"');
assert.sameValue(Math.min.length, 2, 'The value of Math.min.length is expected to be 2');
