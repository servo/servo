// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-244
description: >
    Object.create - one property in 'Properties' is a Function object
    which implements its own [[Get]] method to access the 'get'
    property (8.10.5 step 7.a)
---*/

var funObj = function() {};

funObj.get = function() {
  return "VerifyFunctionObject";
};

var newObj = Object.create({}, {
  prop: funObj
});

assert.sameValue(newObj.prop, "VerifyFunctionObject", 'newObj.prop');
