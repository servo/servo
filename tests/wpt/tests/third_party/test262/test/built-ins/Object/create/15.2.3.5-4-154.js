// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-154
description: >
    Object.create - 'value' property of one property in 'Properties'
    is own data property (8.10.5 step 5.a)
---*/

var newObj = Object.create({}, {
  prop: {
    value: "ownDataProperty"
  }
});

assert.sameValue(newObj.prop, "ownDataProperty", 'newObj.prop');
