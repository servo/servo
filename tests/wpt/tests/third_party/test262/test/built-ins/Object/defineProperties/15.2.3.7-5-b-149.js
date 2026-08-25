// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-149
description: >
    Object.defineProperties - 'writable' property of 'descObj' is own
    accessor property without a get function that overrides an
    inherited accessor property (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var proto = {};

Object.defineProperty(proto, "writable", {
  get: function() {
    return true;
  }
});

var Con = function() {};
Con.prototype = proto;

var descObj = new Con();

Object.defineProperty(descObj, "writable", {
  set: function() {}
});

Object.defineProperties(obj, {
  property: descObj
});

verifyProperty(obj, "property", {
  writable: false,
});
