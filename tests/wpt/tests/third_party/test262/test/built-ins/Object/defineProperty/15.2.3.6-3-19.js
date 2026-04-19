// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-19
description: >
    Object.defineProperty - 'Attributes' is a string primitive (8.10.5
    step 1)
---*/

assert.throws(TypeError, function() {
  Object.defineProperty({}, "property", "abc");
});
