// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-193
description: >
    Object.defineProperties - 'get' property of 'descObj' is own data
    property (8.10.5 step 7.a)
---*/

var obj = {};

var getter = function() {
  return "ownDataProperty";
};

Object.defineProperties(obj, {
  property: {
    get: getter
  }
});

assert.sameValue(obj.property, "ownDataProperty", 'obj.property');
