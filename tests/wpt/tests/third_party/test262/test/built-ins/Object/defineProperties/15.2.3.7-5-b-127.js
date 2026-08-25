// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-127
description: >
    Object.defineProperties - 'descObj' is a String object which
    implements its own [[Get]] method to get 'value' property (8.10.5
    step 5.a)
---*/

var obj = {};

var str = new String("abc");

str.value = "String";

Object.defineProperties(obj, {
  property: str
});

assert.sameValue(obj.property, "String", 'obj.property');
