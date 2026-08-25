// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-49
description: >
    String.prototype.trim - 'this' is a RegExp Object that converts to
    a string
---*/

var regObj = new RegExp(/test/);

assert.sameValue(String.prototype.trim.call(regObj), "/test/", 'String.prototype.trim.call(regObj)');
