// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-3-6
description: >
    Object.defineProperties - enumerable inherited accessor property
    of 'Properties' is not defined in 'O'
---*/

var obj = {};
var proto = {};

Object.defineProperty(proto, "prop", {
  get: function() {
    return {};
  },
  enumerable: true
});

var Con = function() {};
Con.prototype = proto;
var child = new Con();

Object.defineProperties(obj, child);

assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
