// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object
    is set to the original Array prototype object, the one that
    is the initial value of Array.prototype
es5id: 15.4.1_A1.1_T2
description: Array.prototype.toString = Object.prototype.toString
---*/

Array.prototype.toString = Object.prototype.toString;
var x = Array();
assert.sameValue(x.toString(), "[object Array]", 'x.toString() must return "[object Array]"');

Array.prototype.toString = Object.prototype.toString;
var x = Array(0, 1, 2);
assert.sameValue(x.toString(), "[object Array]", 'x.toString() must return "[object Array]"');
