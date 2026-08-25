// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-144
description: >
    Object.defineProperty - 'Attributes' is the Math object that uses
    Object's [[Get]] method to access the 'value' property  (8.10.5
    step 5.a)
---*/

var obj = {};

Math.value = "Math";

Object.defineProperty(obj, "property", Math);

assert.sameValue(obj.property, "Math", 'obj.property');
