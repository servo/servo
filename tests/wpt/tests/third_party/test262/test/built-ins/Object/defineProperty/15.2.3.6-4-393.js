// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-393
description: >
    ES5 Attributes - [[Value]] attribute of data property is a RegExp
    object
---*/

var obj = {};
var regObj = new RegExp();

Object.defineProperty(obj, "prop", {
  value: regObj
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, regObj, 'obj.prop');
assert.sameValue(desc.value, regObj, 'desc.value');
