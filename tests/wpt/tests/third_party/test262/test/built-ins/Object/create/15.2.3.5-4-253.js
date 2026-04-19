// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-253
description: >
    Object.create - one property in 'Properties' is an Error object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var errObj = new Error("error");

errObj.get = function() {
  return "VerifyErrorObject";
};

var newObj = Object.create({}, {
  prop: errObj
});

assert.sameValue(newObj.prop, "VerifyErrorObject", 'newObj.prop');
