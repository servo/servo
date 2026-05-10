// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-27
description: >
    Object.defineProperty - 'enumerable' property in 'Attributes' is
    an inherited accessor property (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

var proto = {};
Object.defineProperty(proto, "enumerable", {
  get: function() {
    return true;
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();

Object.defineProperty(obj, "property", child);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
