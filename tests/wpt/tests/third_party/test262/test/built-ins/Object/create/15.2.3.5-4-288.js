// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-288
description: >
    Object.create - one property in 'Properties' is an Error object
    that uses Object's [[Get]] method to access the 'set' property
    (8.10.5 step 8.a)
---*/

var errObj = new Error("error");
var data = "data";

errObj.set = function(value) {
  data = value;
};

var newObj = Object.create({}, {
  prop: errObj
});

newObj.prop = "overrideData";

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "overrideData", 'data');
