// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-7-1
description: >
    Arguments Object has length as its own property and does not
    invoke the setter defined on Object.prototype.length (Step 7)
includes: [propertyHelper.js]
---*/

var data = "data";
var getFunc = function () {
    return 12;
};

var setFunc = function (value) {
    data = value;
};

Object.defineProperty(Object.prototype, "length", {
    get: getFunc,
    set: setFunc,
    configurable: true
});

var argObj = (function () { return arguments })();

verifyProperty(argObj, "length", {
    value: 0,
    writable: true,
    enumerable: false,
    configurable: true,
});

assert.sameValue(data, "data", 'data');
