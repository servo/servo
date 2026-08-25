// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-a-13
description: Object.isFrozen - 'O' is a Function object
---*/

var obj = function() {};

Object.defineProperty(obj, "property", {
  value: 12,
  writable: true,
  configurable: false
});

Object.preventExtensions(obj);

assert.sameValue(Object.isFrozen(obj), false, 'Object.isFrozen(obj)');
