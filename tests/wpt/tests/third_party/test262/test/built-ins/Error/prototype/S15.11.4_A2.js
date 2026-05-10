// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Class]] property of Error prototype object is
    "Object"
es5id: 15.11.4_A2
description: >
    Getting the value of the internal [[Class]] property using
    Error.prototype.toString() function
---*/

Error.prototype.toString = Object.prototype.toString;
var __tostr = Error.prototype.toString();

assert.sameValue(__tostr, "[object Object]", 'The value of __tostr is expected to be "[object Object]"');
