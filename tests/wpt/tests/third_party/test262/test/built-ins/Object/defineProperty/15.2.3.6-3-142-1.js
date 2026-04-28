// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-142-1
description: >
    Object.defineProperty - 'Attributes' is a Boolean object that uses
    Object's [[Get]] method to access the 'value' property of
    prototype object  (8.10.5 step 5.a)
---*/

var obj = {};

Boolean.prototype.value = "Boolean";
var boolObj = new Boolean(true);

Object.defineProperty(obj, "property", boolObj);

assert.sameValue(obj.property, "Boolean", 'obj.property');
