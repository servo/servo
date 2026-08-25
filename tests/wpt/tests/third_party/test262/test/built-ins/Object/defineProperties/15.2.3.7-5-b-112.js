// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-112
description: >
    Object.defineProperties - 'value' property of 'descObj' is present
    (8.10.5 step 5)
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    value: 300
  }
});

assert.sameValue(obj.property, 300, 'obj.property');
