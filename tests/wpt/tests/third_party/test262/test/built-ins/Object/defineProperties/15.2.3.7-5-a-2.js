// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-2
description: >
    Object.defineProperties - 'P' is own data property that overrides
    enumerable inherited accessor property of 'Properties' is defined
    in 'O'
---*/

var obj = {};
var proto = {};

Object.defineProperty(proto, "prop", {
  get: function() {
    return {
      value: 9
    };
  },
  enumerable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
Object.defineProperty(child, "prop", {
  value: {
    value: 12
  },
  enumerable: true
});
Object.defineProperties(obj, child);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 12, 'obj.prop');
