// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For non-configurable properties, step 9a of
    [[DefineOwnProperty]] rejects changing the kind of a property.
es5id: 15.2.3.6-4-12
description: >
    Object.defineProperty throws TypeError when changing
    non-configurable data properties to accessor properties
---*/

var o = {};

// create a data valued property; all other attributes default to false.
var d1 = {
  value: 101,
  configurable: false
};
Object.defineProperty(o, "foo", d1);

// changing "foo" to be an accessor should fail, since [[Configurable]]
// on the original property will be false.

// dummy getter
var getter = function() {
  return 1;
}

var desc = {
  get: getter
};
assert.throws(TypeError, function() {
  Object.defineProperty(o, "foo", desc);
});
// the property should remain a data valued property.
var d2 = Object.getOwnPropertyDescriptor(o, "foo");
assert.sameValue(d2.value, 101, 'd2.value');
assert.sameValue(d2.writable, false, 'd2.writable');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
assert.sameValue(d2.configurable, false, 'd2.configurable');
