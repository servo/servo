// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-177
description: >
    Object.create - one property in 'Properties' is the global object
    that uses Object's [[Get]] method to access the 'value' property
    (8.10.5 step 5.a)
---*/

this.value = "GlobalValue";

var newObj = Object.create({}, {
  prop: this
});

assert.sameValue(newObj.prop, "GlobalValue", 'newObj.prop');
