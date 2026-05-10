// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-132
description: >
    Object.defineProperty - 'value' property in 'Attributes' is own
    accessor property  (8.10.5 step 5.a)
---*/

var obj = {};

var attr = {};
Object.defineProperty(attr, "value", {
  get: function() {
    return "ownAccessorProperty";
  }
});

Object.defineProperty(obj, "property", attr);

assert.sameValue(obj.property, "ownAccessorProperty", 'obj.property');
