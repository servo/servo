// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-70
description: >
    Object.defineProperties - 'configurable' property of 'descObj' is
    own accessor property without a get function that overrides an
    inherited accessor property (8.10.5 step 4.a)
includes: [propertyHelper.js]
---*/


var obj = {};
var proto = {};
Object.defineProperty(proto, "configurable", {
  get: function() {
    return true;
  }
});

var Con = function() {};
Con.prototype = proto;
var descObj = new Con();

Object.defineProperty(descObj, "configurable", {
  set: function() {}
});

Object.defineProperties(obj, {
  prop: descObj
});

verifyProperty(obj, "prop", {
  configurable: false,
});
