// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-11-b-1
description: >
    Arguments Object has index property '0' as its own property, it
    shoulde be writable, enumerable, configurable and does not invoke
    the setter defined on Object.prototype[0] (Step 11.b)
includes: [propertyHelper.js]
---*/

var data = "data";
var getFunc = function () {
    return data;
};

var setFunc = function (value) {
    data = value;
};

Object.defineProperty(Object.prototype, "0", {
    get: getFunc,
    set: setFunc,
    configurable: true
});

var argObj = (function () { return arguments })(1);

verifyProperty(argObj, "0", {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true,
});

assert.sameValue(data, "data", 'data');
