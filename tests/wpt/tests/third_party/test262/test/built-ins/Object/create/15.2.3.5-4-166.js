// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-166
description: >
    Object.create - one property in 'Properties' is an Array object
    that uses Object's [[Get]] method to access the 'value' property
    (8.10.5 step 5.a)
---*/

var arr = [1, 2, 3];

arr.value = "ArrValue";

var newObj = Object.create({}, {
  prop: arr
});

assert.sameValue(newObj.prop, "ArrValue", 'newObj.prop');
