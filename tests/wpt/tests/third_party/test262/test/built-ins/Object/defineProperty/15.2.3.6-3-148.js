// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-148
description: >
    Object.defineProperty - 'Attributes' is an Error object that uses
    Object's [[Get]] method to access the 'value' property  (8.10.5
    step 5.a)
---*/

var obj = {};

var errObj = new Error();

errObj.value = "Error";

Object.defineProperty(obj, "property", errObj);

assert.sameValue(obj.property, "Error", 'obj.property');
