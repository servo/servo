// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-8
description: >
    String.prototype.trim - argument 'this' is a number that converts
    to a string (value is negative number)
---*/

assert.sameValue(String.prototype.trim.call(-20), "-20", 'String.prototype.trim.call(-20)');
