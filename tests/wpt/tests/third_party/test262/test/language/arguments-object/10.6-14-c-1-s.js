// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-14-c-1-s
description: >
    [[Enumerable]] attribute value in 'callee' is false
includes: [propertyHelper.js]
---*/

var argObj = function () {
    return arguments;
} ();

verifyProperty(argObj, "callee", {
    enumerable: false,
});
