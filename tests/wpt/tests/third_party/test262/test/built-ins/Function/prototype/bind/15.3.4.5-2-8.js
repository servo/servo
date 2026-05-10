// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    15.3.4.5 step 2 specifies that a TypeError must be thrown if the Target
    is not callable.
es5id: 15.3.4.5-2-8
description: Function.prototype.bind allows Target to be a constructor (Array)
---*/

var bac = Array.bind(null);
var a = bac(42);
a.prop = "verifyPropertyExist";
a[41] = 41;

assert.sameValue(a.prop, "verifyPropertyExist", 'a.prop');
assert.sameValue(a[41], 41, 'a[41]');
assert.sameValue(a.length, 42, 'a.length');
