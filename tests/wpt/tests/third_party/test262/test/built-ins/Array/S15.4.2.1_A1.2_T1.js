// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The [[Class]] property of the newly constructed object is set to "Array"
es5id: 15.4.2.1_A1.2_T1
description: Checking use Object.prototype.toString
---*/

var x = new Array();
x.getClass = Object.prototype.toString;
assert.sameValue(x.getClass(), "[object Array]", 'x.getClass() must return "[object Array]"');

var x = new Array(0, 1, 2);
x.getClass = Object.prototype.toString;
assert.sameValue(x.getClass(), "[object Array]", 'x.getClass() must return "[object Array]"');
