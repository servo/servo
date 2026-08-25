// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.4
description: >
    Throws a TypeError exception if boolean trap result is true and target is
    extensible.
info: |
    [[PreventExtensions]] ( )

    ...
    10. If booleanTrapResult is true, then
        ...
        c. If targetIsExtensible is true, throw a TypeError exception.
    11. Return booleanTrapResult.
    ...
features: [Proxy]
---*/

var p = new Proxy({}, {
  preventExtensions: function(t) {
    return true;
  }
});

assert.throws(TypeError, function() {
  Object.preventExtensions(p);
});
