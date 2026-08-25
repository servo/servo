// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For non-configurable properties, step 11.a.i
    of [[DefineOwnProperty]] rejects changing the setter if present.
es5id: 15.2.3.6-4-18
description: >
    Object.defineProperty throws TypeError when changing setter of
    non-configurable accessor properties(8.12.9 step 11.a.i)
---*/

var o = {};

// create an accessor property; all other attributes default to false.
// dummy getter
var getter = function() {
  return 1;
}
var d1 = {
  get: getter
};
Object.defineProperty(o, "foo", d1);

// now, trying to change the setter should fail, since [[Configurable]]
// on the original property will be false.
var setter = function(x) {};
var desc = {
  set: setter
};
assert.throws(TypeError, function() {
  Object.defineProperty(o, "foo", desc);
});
// the property should remain unchanged.
var d2 = Object.getOwnPropertyDescriptor(o, "foo");
assert.sameValue(d2.get, getter, 'd2.get');
assert.sameValue(d2.configurable, false, 'd2.configurable');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
