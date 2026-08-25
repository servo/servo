// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-172
description: >
    Object.create - one property in 'Properties' is a RegExp object
    that uses Object's [[Get]] method to access the 'value' property
    (8.10.5 step 5.a)
---*/

var regObj = new RegExp();

regObj.value = "RegExpValue";

var newObj = Object.create({}, {
  prop: regObj
});

assert.sameValue(newObj.prop, "RegExpValue", 'newObj.prop');
