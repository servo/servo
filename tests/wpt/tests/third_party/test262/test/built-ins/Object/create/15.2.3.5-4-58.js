// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-58
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is an inherited accessor property without a get
    function (8.10.5 step 3.a)
---*/

var proto = {};
var accessed = false;

Object.defineProperty(proto, "enumerable", {
  set: function() {}
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var descObj = new ConstructFun();

var newObj = Object.create({}, {
  prop: descObj
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert.sameValue(accessed, false, 'accessed');
