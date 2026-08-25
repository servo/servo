// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-2-a-7
description: >
    Object.isFrozen - 'P' is own accessor property without a get
    function
---*/

var obj = {};
Object.defineProperty(obj, "foo", {
  set: function() {},
  configurable: true
});

Object.preventExtensions(obj);

assert.sameValue(Object.isFrozen(obj), false, 'Object.isFrozen(obj)');
