// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-241
description: >
    Object.defineProperty - 'set' property in 'Attributes' is own
    accessor property (8.10.5 step 8.a)
---*/

var obj = {};
var data = "data";
var attributes = {};
Object.defineProperty(attributes, "set", {
  get: function() {
    return function(value) {
      data = value;
    };
  }
});

Object.defineProperty(obj, "property", attributes);
obj.property = "ownAccessorProperty";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data, "ownAccessorProperty", 'data');
