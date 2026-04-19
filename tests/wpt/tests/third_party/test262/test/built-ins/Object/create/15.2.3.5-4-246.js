// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-246
description: >
    Object.create - one property in 'Properties' is a String object
    that uses Object's [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var strObj = new String("abc");

strObj.get = function() {
  return "VerifyStringObject";
};

var newObj = Object.create({}, {
  prop: strObj
});

assert.sameValue(newObj.prop, "VerifyStringObject", 'newObj.prop');
