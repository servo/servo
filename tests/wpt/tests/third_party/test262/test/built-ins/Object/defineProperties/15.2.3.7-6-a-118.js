// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-118
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    TypeError is thrown when 'desc' is accessor descriptor (15.4.5.1
    step 3.a.i)
---*/

var arr = [];
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    length: {
      get: function() {
        return 2;
      }
    }
  });
});
assert.sameValue(arr.length, 0, 'arr.length');
