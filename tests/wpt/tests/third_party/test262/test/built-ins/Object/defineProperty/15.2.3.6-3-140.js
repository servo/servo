// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-140
description: >
    Object.defineProperty - 'Attributes' is an Array object that uses
    Object's [[Get]] method to access the 'value' property  (8.10.5
    step 5.a)
---*/

var obj = {};

var arrObj = [1, 2, 3];

arrObj.value = "Array";

Object.defineProperty(obj, "property", arrObj);

assert.sameValue(obj.property, "Array", 'obj.property');
