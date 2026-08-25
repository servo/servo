// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Boolean.prototype has the attribute ReadOnly
esid: sec-boolean.prototype
description: Checking if varying the Boolean.prototype property fails
includes: [propertyHelper.js]
---*/

// CHECK#1
var x = Boolean.prototype;
verifyNotWritable(Boolean, "prototype", null, 1);
assert.sameValue(Boolean.prototype, x, 'The value of Boolean.prototype is expected to equal the value of x');
