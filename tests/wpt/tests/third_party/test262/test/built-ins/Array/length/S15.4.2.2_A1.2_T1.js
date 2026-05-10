// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
info: The [[Class]] property of the newly constructed object is set to "Array"
es5id: 15.4.2.2_A1.2_T1
description: Checking use Object.prototype.toString
---*/

var x = new Array(0);
assert.sameValue(Object.prototype.toString.call(x), "[object Array]", 'Object.prototype.toString.call(new Array(0)) must return "[object Array]"');
