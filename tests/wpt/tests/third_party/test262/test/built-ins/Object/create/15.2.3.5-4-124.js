// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-124
description: >
    Object.create - one property in 'Properties' is the global object
    that uses Object's [[Get]] method to access the 'configurable'
    property (8.10.5 step 4.a)
---*/

this.configurable = true;

var newObj = Object.create({}, {
  prop: this
});

var result1 = newObj.hasOwnProperty("prop");
delete newObj.prop;
var result2 = newObj.hasOwnProperty("prop");

assert.sameValue(result1, true, 'result1');
assert.sameValue(result2, false, 'result2');
