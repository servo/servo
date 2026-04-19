// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O to define the property. For non-configurable properties, step 11.a.ii
    of [[DefineOwnProperty]] permits setting a getter if absent.
es5id: 15.2.3.6-4-21
description: >
    Object.defineProperty permits setting a getter (if absent) of
    non-configurable accessor properties(8.12.9 step 11.a.ii)
---*/

var o = {};

// create an accessor property; all other attributes default to false.
// dummy setter
var setter = function(x) {}
var d1 = {
  set: setter
};
Object.defineProperty(o, "foo", d1);

// now, trying to set the getter should succeed even though [[Configurable]]
// on the original property will be false. Existing values of need to be preserved.
var getter = undefined;
var desc = {
  get: getter
};

Object.defineProperty(o, "foo", desc);
var d2 = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(d2.get, getter, 'd2.get');
assert.sameValue(d2.set, setter, 'd2.set');
assert.sameValue(d2.configurable, false, 'd2.configurable');
assert.sameValue(d2.enumerable, false, 'd2.enumerable');
