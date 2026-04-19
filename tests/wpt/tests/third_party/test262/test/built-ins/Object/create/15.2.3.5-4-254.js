// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-254
description: >
    Object.create - one property in 'Properties' is an Arguments
    object which implements its own [[Get]] method to access the 'get'
    property (8.10.5 step 7.a)
---*/

var argObj = (function() {
  return arguments;
})();

argObj.get = function() {
  return "VerifyArgumentsObject";
};

var newObj = Object.create({}, {
  prop: argObj
});

assert.sameValue(newObj.prop, "VerifyArgumentsObject", 'newObj.prop');
