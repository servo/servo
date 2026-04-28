// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. Step 7b of [[DefineOwnProperty]] rejects if
    current.[[Enumerable]] and desc.[[Enumerable]] are the boolean negations
    of each other.
es5id: 15.2.3.6-4-10
description: >
    Object.defineProperty throws TypeError when changing
    [[Enumerable]] from false to true on non-configurable accessor
    properties
---*/

var o = {};

// create an accessor property; all other attributes default to false.
// dummy getter
var getter = function() {
  return 1;
}
var d1 = {
  get: getter,
  enumerable: false,
  configurable: false
};
Object.defineProperty(o, "foo", d1);

// now, setting enumerable to true should fail, since [[Configurable]]
// on the original property will be false.
var desc = {
  get: getter,
  enumerable: true
};
assert.throws(TypeError, function() {
  Object.defineProperty(o, "foo", desc);
});
// the property should remain unchanged.
var d2 = Object.getOwnPropertyDescriptor(o, "foo");
assert.sameValue(d2.get, getter, 'd2.get');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
assert.sameValue(d2.configurable, false, 'd2.configurable');
