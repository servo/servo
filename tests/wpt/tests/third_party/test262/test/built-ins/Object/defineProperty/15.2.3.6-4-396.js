// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-396
description: ES5 Attributes - [[Value]] attribute of data property is NaN
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: NaN
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.prop !== obj.prop, 'obj.prop !== obj.prop !== true');
assert(desc.value !== desc.value, 'desc.value !== desc.value !== true');
