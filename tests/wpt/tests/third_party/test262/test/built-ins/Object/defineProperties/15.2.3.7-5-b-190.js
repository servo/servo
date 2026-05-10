// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-190
description: >
    Object.defineProperties - value of 'writable' property of
    'descObj' is  new Boolean(false) which is treated as true value
    (8.10.5 step 6.b)
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    writable: new Boolean(false)
  }
});

obj.property = "isWritable";

assert.sameValue(obj.property, "isWritable", 'obj.property');
