// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For configurable properties, step 9c of
    [[DefineOwnProperty]] permits changing the kind of a property.
es5id: 15.2.3.6-4-15
description: >
    Object.defineProperty permits changing accessor property to data
    property for configurable properties
---*/

var o = {};

// define an accessor property
// dummy getter
var getter = function() {
  return 1;
}
var d1 = {
  get: getter,
  configurable: true
};
Object.defineProperty(o, "foo", d1);

// changing "foo" to be a data valued property should succeed, since
// [[Configurable]] on the original property will be true. Existing
// values of [[Configurable]] and [[Enumerable]] need to be preserved
// and the rest need to be set to their default values.
var desc = {
  value: 101
};
Object.defineProperty(o, "foo", desc);
var d2 = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(d2.value, 101, 'd2.value');
assert.sameValue(d2.writable, false, 'd2.writable');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
assert.sameValue(d2.configurable, true, 'd2.configurable');
