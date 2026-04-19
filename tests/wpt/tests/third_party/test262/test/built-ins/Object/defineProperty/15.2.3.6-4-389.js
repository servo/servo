// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-389
description: >
    ES5 Attributes - [[Value]] attribute of data property is a Boolean
    Object
---*/

var obj = {};
var boolObj = new Boolean();

Object.defineProperty(obj, "prop", {
  value: boolObj
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, boolObj, 'obj.prop');
assert.sameValue(desc.value, boolObj, 'desc.value');
