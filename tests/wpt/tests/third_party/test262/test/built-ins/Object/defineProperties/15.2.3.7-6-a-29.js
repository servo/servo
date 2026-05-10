// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-29
description: >
    Object.defineProperties - 'P' doesn't exist in 'O', test
    [[Enumerable]] of 'P' is set as false value if absent in data
    descriptor 'desc' (8.12.9 step 4.a.i)
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {
    value: 1001
  }
});

for (var prop in obj) {
  if (obj.hasOwnProperty(prop)) {
    assert.notSameValue(prop, "prop", 'prop');
  }
}
