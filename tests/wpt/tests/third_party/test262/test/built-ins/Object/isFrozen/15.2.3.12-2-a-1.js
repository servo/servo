// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-a-1
description: Object.isFrozen - 'P' is own data property
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 12,
  writable: true,
  configurable: false
});

Object.preventExtensions(obj);

assert.sameValue(Object.isFrozen(obj), false, 'Object.isFrozen(obj)');
