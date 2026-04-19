// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-57
description: >
    Object.defineProperties - both desc.[[Get]] and P.[[Get]] are two
    objects which refer to the same object (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_Func() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: get_Func
});

Object.defineProperties(obj, {
  foo: {
    get: get_Func
  }
});

assert.sameValue(obj.foo, 10);

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
