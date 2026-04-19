// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-21
description: >
    Object.create - an enumerable inherited accessor property in
    'Properties' is not defined in 'obj' (15.2.3.7 step 3)
---*/

var proto = {};

Object.defineProperty(proto, "prop", {
  get: function() {
    return {};
  },
  enumerable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var child = new ConstructFun();

var newObj = Object.create({}, child);

assert.sameValue(newObj.hasOwnProperty("prop"), false, 'newObj.hasOwnProperty("prop")');
