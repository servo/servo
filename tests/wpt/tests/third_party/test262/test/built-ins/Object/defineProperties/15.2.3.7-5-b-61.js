// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-61
description: >
    Object.defineProperties - 'configurable' property of 'descObj' is
    own data property (8.10.5 step 4.a)
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {
    configurable: true
  }
});

var result1 = obj.hasOwnProperty("prop");
delete obj.prop;
var result2 = obj.hasOwnProperty("prop");

assert.sameValue(result1, true, 'result1');
assert.sameValue(result2, false, 'result2');
