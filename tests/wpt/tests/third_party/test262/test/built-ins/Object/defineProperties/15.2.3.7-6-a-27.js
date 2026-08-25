// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-27
description: >
    Object.defineProperties - 'P' doesn't exist in 'O', test [[Value]]
    of 'P' is set as undefined value if absent in data descriptor
    'desc' (8.12.9 step 4.a.i)
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {
    writable: true
  }
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc.hasOwnProperty("value"), 'desc.hasOwnProperty("value") !== true');
assert.sameValue(typeof desc.value, "undefined", 'typeof desc.value');
assert(desc.hasOwnProperty("writable"), 'desc.hasOwnProperty("writable") !== true');
assert.sameValue(desc.writable, true, 'desc.writable');
assert(desc.hasOwnProperty("configurable"), 'desc.hasOwnProperty("configurable") !== true');
assert.sameValue(desc.configurable, false, 'desc.configurable');
assert(desc.hasOwnProperty("enumerable"), 'desc.hasOwnProperty("enumerable") !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
