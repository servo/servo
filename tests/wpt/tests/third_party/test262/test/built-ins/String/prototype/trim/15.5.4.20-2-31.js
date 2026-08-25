// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-31
description: >
    String.prototype.trim - argument 'this' is a string(value is
    'null')
---*/

assert.sameValue(String.prototype.trim.call("null"), "null", 'String.prototype.trim.call("null")');
