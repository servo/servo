// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-234
description: >
    Object.create - 'get' property of one property in 'Properties' is
    an inherited data property (8.10.5 step 7.a)
---*/

var proto = {
  get: function() {
    return "inheritedDataProperty";
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var descObj = new ConstructFun();

var newObj = Object.create({}, {
  prop: descObj
});

assert.sameValue(newObj.prop, "inheritedDataProperty", 'newObj.prop');
