// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-27
description: >
    String.prototype.trim - argument 'this' is a number that converts
    to a string (value is 123.1234567)
---*/

assert.sameValue(String.prototype.trim.call(123.1234567), "123.1234567", 'String.prototype.trim.call(123.1234567)');
