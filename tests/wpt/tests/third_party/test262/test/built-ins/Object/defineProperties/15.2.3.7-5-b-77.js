// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-77
description: >
    Object.defineProperties - 'descObj' is the Math object which
    implements its own [[Get]] method to get 'configurable' property
    (8.10.5 step 4.a)
---*/

var obj = {};

Math.configurable = true;

Object.defineProperties(obj, {
  prop: Math
});

var result1 = obj.hasOwnProperty("prop");
delete obj.prop;
var result2 = obj.hasOwnProperty("prop");

assert.sameValue(result1, true, 'result1');
assert.sameValue(result2, false, 'result2');
