// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-180
description: >
    Object.create - 'writable' property of one property in
    'Properties' is own data property (8.10.5 step 6.a)
---*/

var newObj = Object.create({}, {
  prop: {
    writable: true
  }
});

var beforeWrite = (newObj.hasOwnProperty("prop") && typeof(newObj.prop) === "undefined");

newObj.prop = "isWritable";

var afterWrite = (newObj.prop === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
