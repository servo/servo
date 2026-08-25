// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-207
description: >
    Object.defineProperty - 'get' property in 'Attributes' is own data
    property (8.10.5 step 7.a)
---*/

var obj = {};
var attributes = {
  get: function() {
    return "ownDataProperty";
  }
};

Object.defineProperty(obj, "property", attributes);

assert.sameValue(obj.property, "ownDataProperty", 'obj.property');
