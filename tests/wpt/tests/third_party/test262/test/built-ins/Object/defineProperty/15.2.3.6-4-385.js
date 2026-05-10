// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-385
description: >
    ES5 Attributes - [[Value]] attribute of data property is a generic
    object
---*/

var obj = {};
var tempObj = {
  testproperty: 100
};

Object.defineProperty(obj, "prop", {
  value: tempObj
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, tempObj, 'obj.prop');
assert.sameValue(desc.value, tempObj, 'desc.value');
