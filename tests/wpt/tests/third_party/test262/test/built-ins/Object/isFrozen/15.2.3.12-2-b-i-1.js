// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-b-i-1
description: >
    Object.isFrozen returns false if 'O' contains own writable data
    property
---*/

var obj = {};
Object.defineProperty(obj, "foo", {
  value: 20,
  writable: true,
  configurable: false
});
Object.preventExtensions(obj);

assert.sameValue(Object.isFrozen(obj), false, 'Object.isFrozen(obj)');
