// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For non-configurable properties, step 9a of
    [[DefineOwnProperty]] rejects changing the kind of a property.
es5id: 15.2.3.6-4-13
description: >
    Object.defineProperty throws TypeError when changing
    non-configurable accessor properties to data properties
---*/

var o = {};

// create an accessor property; all other attributes default to false.

// dummy getter
var getter = function() {
  return 1;
}
var d1 = {
  get: getter,
  configurable: false
};
Object.defineProperty(o, "foo", d1);

// changing "foo" to be a data property should fail, since [[Configurable]]
// on the original property will be false.
var desc = {
  value: 101
};
assert.throws(TypeError, function() {
  Object.defineProperty(o, "foo", desc);
});
// the property should remain an accessor property.
var d2 = Object.getOwnPropertyDescriptor(o, "foo");
assert.sameValue(d2.get, getter, 'd2.get');
assert.sameValue(d2.configurable, false, 'd2.configurable');
