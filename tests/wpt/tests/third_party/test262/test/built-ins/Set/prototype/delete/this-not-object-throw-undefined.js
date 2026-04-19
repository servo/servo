// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.delete
description: >
    Set.prototype.delete ( value )

    1. Let S be the this value.
    2. If Type(S) is not Object, throw a TypeError exception.

---*/

assert.throws(TypeError, function() {
  Set.prototype.delete.call(undefined, 1);
});

assert.throws(TypeError, function() {
  var s = new Set();
  s.delete.call(undefined, 1);
});
