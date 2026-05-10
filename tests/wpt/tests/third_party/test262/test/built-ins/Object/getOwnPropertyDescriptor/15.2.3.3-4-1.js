// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-1
description: >
    Object.getOwnPropertyDescriptor returns an object representing a
    data desc for valid data valued properties
---*/

var o = {};
o["foo"] = 101;

var desc = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(desc.value, 101, 'desc.value');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
assert.sameValue(desc.hasOwnProperty("get"), false, 'desc.hasOwnProperty("get")');
assert.sameValue(desc.hasOwnProperty("set"), false, 'desc.hasOwnProperty("set")');
