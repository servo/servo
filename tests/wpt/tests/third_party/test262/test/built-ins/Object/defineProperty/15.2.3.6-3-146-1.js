// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-146-1
description: >
    Object.defineProperty - 'Attributes' is a RegExp object that uses
    Object's [[Get]] method to access the 'value' property of
    prototype object  (8.10.5 step 5.a)
---*/

var obj = {};

RegExp.prototype.value = "RegExp";
var regObj = new RegExp();

Object.defineProperty(obj, "property", regObj);

assert.sameValue(obj.property, "RegExp", 'obj.property');
