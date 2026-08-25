// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-a-2
description: >
    Object.keys - 'writable' attribute of element of returned array is
    correct
---*/

var obj = {
  prop1: 100
};

var array = Object.keys(obj);

array[0] = "isWritable";

var desc = Object.getOwnPropertyDescriptor(array, "0");

assert.sameValue(array[0], "isWritable", 'array[0]');
assert(desc.hasOwnProperty("writable"), 'desc.hasOwnProperty("writable") !== true');
assert.sameValue(desc.writable, true, 'desc.writable');
