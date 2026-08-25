// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-26
description: >
    Object.defineProperties - 'P' doesn't exist in 'O', test 'P' is
    defined as data property when 'desc' is generic descriptor (8.12.9
    step 4.a)
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {
    configurable: true,
    enumerable: true
  }
});
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc.hasOwnProperty("value"), 'desc.hasOwnProperty("value") !== true');
assert.sameValue(typeof desc.value, "undefined", 'typeof desc.value');
assert(desc.hasOwnProperty("writable"), 'desc.hasOwnProperty("writable") !== true');
assert.sameValue(desc.writable, false, 'desc.writable');
assert(desc.hasOwnProperty("configurable"), 'desc.hasOwnProperty("configurable") !== true');
assert.sameValue(desc.configurable, true, 'desc.configurable');
assert(desc.hasOwnProperty("enumerable"), 'desc.hasOwnProperty("enumerable") !== true');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.hasOwnProperty("get"), false, 'desc.hasOwnProperty("get")');
assert.sameValue(desc.hasOwnProperty("set"), false, 'desc.hasOwnProperty("set")');
