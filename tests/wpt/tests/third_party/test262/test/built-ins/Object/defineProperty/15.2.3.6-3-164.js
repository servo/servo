// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-164
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is an
    inherited accessor property without a get function  (8.10.5 step
    6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var proto = {};
Object.defineProperty(proto, "writable", {
  set: function() {}
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();

Object.defineProperty(obj, "property", child);

verifyProperty(obj, "property", {
  writable: false,
});
