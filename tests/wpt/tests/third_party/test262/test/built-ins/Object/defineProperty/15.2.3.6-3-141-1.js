// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-141-1
description: >
    Object.defineProperty - 'Attributes' is a String object that uses
    Object's [[Get]] method to access the 'value' property of
    prototype object  (8.10.5 step 5.a)
---*/

var obj = {};

String.prototype.value = "String";
var strObj = new String("abc");

Object.defineProperty(obj, "property", strObj);

assert.sameValue(obj.property, "String", 'obj.property');
