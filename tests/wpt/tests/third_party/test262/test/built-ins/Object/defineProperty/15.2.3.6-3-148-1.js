// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-148-1
description: >
    Object.defineProperty - 'Attributes' is an Error object that uses
    Object's [[Get]] method to access the 'value' property of
    prototype object  (8.10.5 step 5.a)
---*/

var obj = {};

Error.prototype.value = "Error";
var errObj = new Error();

Object.defineProperty(obj, "property", errObj);

assert.sameValue(obj.property, "Error", 'obj.property');
