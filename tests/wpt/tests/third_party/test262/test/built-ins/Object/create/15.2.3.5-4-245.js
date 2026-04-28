// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-245
description: >
    Object.create - one property in 'Properties' is an Array object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var arrayObj = [1, 2, 3];

arrayObj.get = function() {
  return "VerifyArrayObject";
};

var newObj = Object.create({}, {
  prop: arrayObj
});

assert.sameValue(newObj.prop, "VerifyArrayObject", 'newObj.prop');
