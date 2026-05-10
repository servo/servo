// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-172
description: >
    Object.defineProperties - value of 'writable' property of
    'descObj' is positive number (8.10.5 step 6.b)
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    writable: 123
  }
});

obj.property = "isWritable";

assert.sameValue(obj.property, "isWritable", 'obj.property');
