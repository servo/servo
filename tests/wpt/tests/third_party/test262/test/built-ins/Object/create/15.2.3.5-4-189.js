// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-189
description: >
    Object.create - 'writable' property of one property in
    'Properties' is own accessor property without a get function,
    which overrides an inherited accessor property (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var proto = {};

Object.defineProperty(proto, "writable", {
  get: function() {
    return true;
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var descObj = new ConstructFun();

Object.defineProperty(descObj, "writable", {
  set: function() {}
});

var newObj = Object.create({}, {
  prop: descObj
});

verifyProperty(newObj, "prop", {
  value: undefined,
  writable: false,
});
