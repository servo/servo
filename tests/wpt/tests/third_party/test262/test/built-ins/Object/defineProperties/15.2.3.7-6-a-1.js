// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-1
description: >
    Object.defineProperties - 'P' is own existing data property
    (8.12.9 step 1 )
---*/

var obj = {};
Object.defineProperty(obj, "prop", {
  value: 11,
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperties(obj, {
    prop: {
      value: 12,
      configurable: true
    }
  });
});
