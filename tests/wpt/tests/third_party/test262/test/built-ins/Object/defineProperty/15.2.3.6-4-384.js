// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-384
description: ES5 Attributes - [[Value]] attribute of data property is a boolean
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: false
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, false, 'obj.prop');
assert.sameValue(desc.value, false, 'desc.value');
