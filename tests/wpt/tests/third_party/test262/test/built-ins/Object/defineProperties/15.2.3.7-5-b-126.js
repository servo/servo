// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-126
description: >
    Object.defineProperties - 'descObj' is an Array object which
    implements its own [[Get]] method to get 'value' property (8.10.5
    step 5.a)
---*/

var obj = {};

var arr = [1, 2, 3];

arr.value = "Array";

Object.defineProperties(obj, {
  property: arr
});

assert.sameValue(obj.property, "Array", 'obj.property');
