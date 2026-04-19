// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-153
description: >
    Object.create - 'value' property of one property in 'Properties'
    is not present (8.10.5 step 5)
---*/

var newObj = Object.create({}, {
  prop: {}
});

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
assert.sameValue(typeof(newObj.prop), "undefined", 'typeof (newObj.prop)');
