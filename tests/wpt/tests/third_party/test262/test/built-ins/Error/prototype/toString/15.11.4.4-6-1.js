// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.11.4.4-6-1
description: >
    Error.prototype.toString - 'Error' is returned when 'name' is
    absent and empty string is returned when 'msg' is undefined
---*/

var errObj = new Error();

assert.sameValue(errObj.toString(), "Error", 'errObj.toString()');
