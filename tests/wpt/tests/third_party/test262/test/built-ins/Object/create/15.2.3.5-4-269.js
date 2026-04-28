// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-269
description: >
    Object.create - 'set' property of one property in 'Properties' is
    an inherited data property (8.10.5 step 8.a)
---*/

var data = "data";
var proto = {
  set: function(value) {
    data = value;
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var child = new ConstructFun();

var newObj = Object.create({}, {
  prop: child
});

var hasProperty = newObj.hasOwnProperty("prop");

newObj.prop = "overrideData";

assert(hasProperty, 'hasProperty !== true');
assert.sameValue(data, "overrideData", 'data');
