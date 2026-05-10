// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For non-configurable properties, step 11.a.ii
    of [[DefineOwnProperty]] rejects changing the getter if present.
es5id: 15.2.3.6-4-20
description: >
    Object.defineProperty throws TypeError when changing getter (if
    present) of non-configurable accessor properties(8.12.9 step
    11.a.ii)
---*/

var o = {};

// create an accessor property; all other attributes default to false.
// dummy getter/setter
var getter = function() {
  return 1;
}
var d1 = {
  get: getter,
  configurable: false
};
Object.defineProperty(o, "foo", d1);

// now, trying to change the setter should fail, since [[Configurable]]
// on the original property will be false.
var desc = {
  get: undefined
};
assert.throws(TypeError, function() {
  Object.defineProperty(o, "foo", desc);
});
var d2 = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(d2.get, getter, 'd2.get');
assert.sameValue(d2.configurable, false, 'd2.configurable');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
