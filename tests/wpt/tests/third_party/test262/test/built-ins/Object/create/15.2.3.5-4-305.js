// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-305
description: >
    Object.create defines a data property when one property in
    'Properties' is generic descriptor (8.12.9 step 4.a)
---*/

var newObj = Object.create({}, {
  prop: {
    enumerable: true
  }
});

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
