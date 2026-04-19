// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-356
description: >
    ES5 Attributes - property ([[Writable]] is false, [[Enumerable]]
    is true, [[Configurable]] is true) is deletable
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: true,
  configurable: true
});
var beforeDelete = obj.hasOwnProperty("prop");
delete obj.prop;
var afterDelete = obj.hasOwnProperty("prop");

assert(beforeDelete, 'beforeDelete !== true');
assert.sameValue(afterDelete, false, 'afterDelete');
