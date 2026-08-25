// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-194
description: >
    Object.create - one property in 'Properties' is a Boolean object
    that uses Object's [[Get]] method to access the 'writable'
    property (8.10.5 step 6.a)
---*/

var booleanObj = new Boolean(false);

booleanObj.writable = true;

var newObj = Object.create({}, {
  prop: booleanObj
});

var beforeWrite = (newObj.hasOwnProperty("prop") && typeof(newObj.prop) === "undefined");

newObj.prop = "isWritable";

var afterWrite = (newObj.prop === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
