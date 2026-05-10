// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-195
description: >
    Object.create - one property in 'Properties' is a Number object
    that uses Object's [[Get]] method to access the 'writable'
    property (8.10.5 step 6.a)
---*/

var numObj = new Number(123);

numObj.writable = true;

var newObj = Object.create({}, {
  prop: numObj
});

var beforeWrite = (newObj.hasOwnProperty("prop") && typeof(newObj.prop) === "undefined");

newObj.prop = "isWritable";

var afterWrite = (newObj.prop === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
