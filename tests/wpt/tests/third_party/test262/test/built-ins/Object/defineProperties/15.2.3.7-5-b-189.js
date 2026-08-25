// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-189
description: >
    Object.defineProperties - value of 'writable' property of
    'descObj' is a string (value is 'false') which is treated as true
    value (8.10.5 step 6.b)
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    writable: "false"
  }
});

obj.property = "isWritable";

assert.sameValue(obj.property, "isWritable", 'obj.property');
