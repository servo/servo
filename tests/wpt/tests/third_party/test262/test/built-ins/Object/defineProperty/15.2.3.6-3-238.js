// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-238
description: >
    Object.defineProperty - 'set' property in 'Attributes' is an
    inherited data property (8.10.5 step 8.a)
---*/

var obj = {};
var data = "data";
var proto = {
  set: function(value) {
    data = value;
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();

Object.defineProperty(obj, "property", child);

obj.property = "overrideData";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data, "overrideData", 'data');
