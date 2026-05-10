// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-17
description: >
    Object.defineProperties - 'enumerable' property of 'descObj'  is
    own accessor property without a get function that overrides an
    inherited accessor property (8.10.5 step 3.a)
---*/

var obj = {};
var proto = {};
var accessed = false;
Object.defineProperty(proto, "enumerable", {
  get: function() {
    return false;
  }
});

var Con = function() {};
Con.prototype = proto;
var descObj = new Con();

Object.defineProperty(descObj, "enumerable", {
  set: function() {}
});

Object.defineProperties(obj, {
  prop: descObj
});
for (var property in obj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert.sameValue(accessed, false, 'accessed');
