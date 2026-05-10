// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    pattern is an object R whose [[Class]] property is "RegExp" and flags
    is not undefined
es5id: 15.10.4.1_A2_T1
description: >
    Checking if execution of "new RegExp(pattern, "i")", where the
    pattern is "/\u0042/i", does not fail
---*/

var regExpObj = new RegExp(/\u0042/i, "i");
assert(regExpObj.ignoreCase);
