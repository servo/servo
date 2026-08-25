// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-146
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is an Error object (8.10.5 step 4.b)
---*/

var newObj = Object.create({}, {
  prop: {
    configurable: new Error()
  }
});

var beforeDeleted = newObj.hasOwnProperty("prop");

delete newObj.prop;

var afterDeleted = newObj.hasOwnProperty("prop");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
