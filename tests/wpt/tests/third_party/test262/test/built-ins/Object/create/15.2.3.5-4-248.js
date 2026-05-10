// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-248
description: >
    Object.create - one property in 'Properties' is a Number object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var numObj = new Number(5);

numObj.get = function() {
  return "VerifyNumberObject";
};

var newObj = Object.create({}, {
  prop: numObj
});

assert.sameValue(newObj.prop, "VerifyNumberObject", 'newObj.prop');
