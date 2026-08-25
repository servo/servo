// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-304
description: >
    Object.create - TypeError is thrown if both 'get' property and
    'writable' property of one property in 'Properties' are present
    (8.10.5 step 9.a)
---*/


assert.throws(TypeError, function() {
  Object.create({}, {
    prop: {
      get: function() {},
      writable: true
    }
  });
});
