// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-6
description: >
    Object.defineProperties - enumerable own accessor property of
    'Properties' without a get function that overrides enumerable
    inherited accessor property of 'Properties' is defined in 'O'
---*/

var obj = {};

var proto = {};

Object.defineProperty(proto, "prop", {
  get: function() {
    return 12;
  },
  enumerable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
Object.defineProperty(child, "prop", {
  get: function() {
    return {
      set: function() {}
    };
  },
  enumerable: true
});
Object.defineProperties(obj, child);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(typeof obj.prop, "undefined", 'typeof obj.prop');
