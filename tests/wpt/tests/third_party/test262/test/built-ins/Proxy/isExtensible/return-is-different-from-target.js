// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.3
description: >
    Throws a TypeError exception if boolean trap result is not the same as
    target.[[IsExtensible]]() result
info: |
    [[IsExtensible]] ( )

    ...
    12. If SameValue(booleanTrapResult, targetResult) is false, throw a
    TypeError exception.
    ...
features: [Proxy]
---*/

var p = new Proxy({}, {
  isExtensible: function(t) {
    return false;
  }
});

assert.throws(TypeError, function() {
  Object.isExtensible(p);
});
