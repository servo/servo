// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-205
description: >
    Object.defineProperty - 'get' property in 'Attributes' is present
    (8.10.5 step 7)
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  get: function() {
    return "present";
  }
});

assert.sameValue(obj.property, "present", 'obj.property');
