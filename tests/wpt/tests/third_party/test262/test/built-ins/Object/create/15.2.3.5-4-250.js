// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-250
description: >
    Object.create - one property in 'Properties' is a RegExp object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var regObj = new RegExp();

regObj.get = function() {
  return "VerifyRegExpObject";
};

var newObj = Object.create({}, {
  prop: regObj
});

assert.sameValue(newObj.prop, "VerifyRegExpObject", 'newObj.prop');
