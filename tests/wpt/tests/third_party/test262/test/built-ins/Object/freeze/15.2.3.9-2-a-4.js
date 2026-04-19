// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-4
description: Object.freeze - 'P' is own accessor property
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  get: function() {
    return 10;
  },
  configurable: true
});

Object.freeze(obj);

verifyProperty(obj, "foo", {
  configurable: false,
});

assert.sameValue(obj.foo, 10);
