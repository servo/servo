// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-13-a-1
description: >
    In non-strict mode, arguments object should have its own 'callee'
    property defined (Step 13.a)
includes: [propertyHelper.js]
flags: [noStrict]
---*/

Object.defineProperty(Object.prototype, "callee", {
    value: 1,
    writable: false,
    configurable: true
});

var argObj = (function () { return arguments })();

assert.sameValue(typeof argObj.callee, "function");

verifyProperty(argObj, "callee", {
    writable: true,
    enumerable: false,
    configurable: true,
});
