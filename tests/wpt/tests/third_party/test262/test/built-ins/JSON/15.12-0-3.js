// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test should be run without any built-ins being added/augmented.
    The name JSON must be bound to an object, and must not support [[Call]].
    step 5 in 11.2.3 should throw a TypeError exception.
es5id: 15.12-0-3
description: JSON must not support the [[Call]] method
---*/

var o = JSON;
assert.throws(TypeError, function() {
  var j = JSON();
});
