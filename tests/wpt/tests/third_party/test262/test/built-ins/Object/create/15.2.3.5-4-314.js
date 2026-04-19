// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-314
description: >
    Object.create - some enumerable own property in 'Properties' is
    empty object (15.2.3.7 step 7)
---*/

var newObj = Object.create({}, {
  foo: {}
});

assert(newObj.hasOwnProperty("foo"), 'newObj.hasOwnProperty("foo") !== true');
