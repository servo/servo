// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-58
description: >
    Object.defineProperties - desc.[[Get]] and P.[[Get]] are two
    objects which refer to the different objects (8.12.9 step 6)
includes: [propertyHelper.js]
---*/

var obj = {};

function get_Func1() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: get_Func1,
  configurable: true
});

function get_Func2() {
  return 20;
}

Object.defineProperties(obj, {
  foo: {
    get: get_Func2
  }
});

assert.sameValue(obj.foo, 20);

var desc = Object.getOwnPropertyDescriptor(obj, "foo");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});

assert.sameValue(typeof(desc.set), "undefined", 'typeof (desc.set)');
assert.sameValue(desc.get, get_Func2, 'desc.get');
