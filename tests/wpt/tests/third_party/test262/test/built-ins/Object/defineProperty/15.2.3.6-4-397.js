// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-397
description: ES5 Attributes - [[Value]] attribute of data property is Infinity
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: Infinity
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, Infinity, 'obj.prop');
assert.sameValue(desc.value, Infinity, 'desc.value');
