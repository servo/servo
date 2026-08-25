// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-47
description: >
    String.prototype.trim - 'this' is a object Object that converts to
    a string
---*/

assert.sameValue(String.prototype.trim.call({}), "[object Object]", 'String.prototype.trim.call({})');
