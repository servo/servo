// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-392
description: >
    ES5 Attributes - [[Value]] attribute of data property is a Date
    object
---*/

var obj = {};
var dateObj = new Date(0);

Object.defineProperty(obj, "prop", {
  value: dateObj
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, dateObj, 'obj.prop');
assert.sameValue(desc.value, dateObj, 'desc.value');
