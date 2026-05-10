// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-247
description: >
    Object.create - one property in 'Properties' is a Boolean object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var boolObj = new Boolean(true);

boolObj.get = function() {
  return "VerifyBooleanObject";
};

var newObj = Object.create({}, {
  prop: boolObj
});

assert.sameValue(newObj.prop, "VerifyBooleanObject", 'newObj.prop');
