// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-191
description: >
    Object.defineProperties - 'get' property of 'descObj' is present
    (8.10.5 step 7)
---*/

var obj = {};

var getter = function() {
  return "present";
};

Object.defineProperties(obj, {
  property: {
    get: getter
  }
});

assert.sameValue(obj.property, "present", 'obj.property');
