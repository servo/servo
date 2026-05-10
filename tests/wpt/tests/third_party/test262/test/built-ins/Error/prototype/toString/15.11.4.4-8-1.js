// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.11.4.4-8-1
description: >
    Error.prototype.toString return the value of 'msg' when 'name' is
    empty string and 'msg' isn't undefined
---*/

var errObj = new Error("ErrorMessage");
errObj.name = "";

assert.sameValue(errObj.toString(), "ErrorMessage", 'errObj.toString()');
