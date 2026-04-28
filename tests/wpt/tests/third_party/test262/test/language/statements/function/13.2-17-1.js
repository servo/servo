// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-17-1
description: >
    Function Object has 'constructor' as its own property, it is not
    enumerable and does not invoke the setter defined on
    Function.prototype.constructor (Step 17)
includes: [propertyHelper.js]
---*/

var desc = Object.getOwnPropertyDescriptor(Object.prototype, "constructor");

var getFunc = function () {
    return 100;
};

var data = "data";
var setFunc = function (value) {
    data = value;
};

Object.defineProperty(Object.prototype, "constructor", {
    get: getFunc,
    set: setFunc,
    configurable: true
});

var fun = function () {};

assert.sameValue(typeof fun.prototype.constructor, "function");

verifyProperty(fun.prototype, "constructor", {
    writable: true,
    enumerable: false,
    configurable: true,
});

assert.sameValue(data, "data", 'data');
