// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test should be run without any built-ins being added/augmented.
    The name JSON must be bound to an object, and must not support [[Construct]].
    step 4 in 11.2.2 should throw a TypeError exception.
es5id: 15.12-0-2
description: JSON must not support the [[Construct]] method
---*/

var o = JSON;
assert.throws(TypeError, function() {
  var j = new JSON();
});
