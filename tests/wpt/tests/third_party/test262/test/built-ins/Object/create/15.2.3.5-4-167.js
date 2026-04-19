// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-167
description: >
    Object.create - one property in 'Properties' is a String object
    that uses Object's [[Get]] method to access the 'value' property
    (8.10.5 step 5.a)
---*/

var str = new String("abc");

str.value = "StrValue";

var newObj = Object.create({}, {
  prop: str
});

assert.sameValue(newObj.prop, "StrValue", 'newObj.prop');
