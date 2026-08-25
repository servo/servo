// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-118
description: >
    Object.defineProperties - 'value' property of 'descObj' is own
    accessor property (8.10.5 step 5.a)
---*/

var obj = {};

var descObj = {};

Object.defineProperty(descObj, "value", {
  get: function() {
    return "ownAccessorProperty";
  }
});

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "ownAccessorProperty", 'obj.property');
