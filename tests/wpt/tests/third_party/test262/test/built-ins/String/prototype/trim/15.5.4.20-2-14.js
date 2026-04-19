// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-14
description: >
    String.prototype.trim - argument 'this' is a number that converts
    to a string (value is 1(following 22 zeros))
---*/

assert.sameValue(String.prototype.trim.call(10000000000000000000000), "1e+22", 'String.prototype.trim.call(10000000000000000000000)');
