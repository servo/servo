// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-32
description: >
    Object.defineProperties - 'desc' is generic descriptor without any
    attribute, test 'P' is defined in 'obj' with all default attribute
    values (8.12.9 step 4.a.i)
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {}
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc.hasOwnProperty("value"), 'desc.hasOwnProperty("value") !== true');
assert.sameValue(typeof desc.value, "undefined", 'typeof desc.value');
assert(desc.hasOwnProperty("writable"), 'desc.hasOwnProperty("writable") !== true');
assert.sameValue(desc.writable, false, 'desc.writable');
assert(desc.hasOwnProperty("configurable"), 'desc.hasOwnProperty("configurable") !== true');
assert.sameValue(desc.configurable, false, 'desc.configurable');
assert(desc.hasOwnProperty("enumerable"), 'desc.hasOwnProperty("enumerable") !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.hasOwnProperty("get"), false, 'desc.hasOwnProperty("get")');
assert.sameValue(desc.hasOwnProperty("set"), false, 'desc.hasOwnProperty("set")');
