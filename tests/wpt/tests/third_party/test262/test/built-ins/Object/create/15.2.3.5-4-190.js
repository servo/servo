// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-190
description: >
    Object.create - 'writable' property of one property in
    'Properties' is an inherited accessor property without a get
    function (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var proto = {
  value: 100
};

Object.defineProperty(proto, "writable", {
  set: function() {}
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var descObj = new ConstructFun();

var newObj = Object.create({}, {
  prop: descObj
});

verifyProperty(newObj, "prop", {
  value: 100,
  writable: false,
});
