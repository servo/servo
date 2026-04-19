// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-105
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is own accessor property (8.10.5 step 4.a)
---*/

var descObj = {};
Object.defineProperty(descObj, "configurable", {
  get: function() {
    return true;
  }
});

var newObj = Object.create({}, {
  prop: descObj
});
var result1 = newObj.hasOwnProperty("prop");
delete newObj.prop;
var result2 = newObj.hasOwnProperty("prop");

assert.sameValue(result1, true, 'result1');
assert.sameValue(result2, false, 'result2');
