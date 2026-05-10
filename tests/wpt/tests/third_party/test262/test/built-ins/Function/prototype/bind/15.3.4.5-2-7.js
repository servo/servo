// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    15.3.4.5 step 2 specifies that a TypeError must be thrown if the Target
    is not callable.
es5id: 15.3.4.5-2-7
description: >
    Function.prototype.bind throws TypeError if the Target is not
    callable (JSON)
---*/


assert.throws(TypeError, function() {
  JSON.bind();
});
