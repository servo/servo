// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.has
description: >
    Set.prototype.has ( value )

    ...
    3. If S does not have a [[SetData]] internal slot, throw a TypeError exception.
    ...

---*/

assert.throws(TypeError, function() {
  Set.prototype.has.call(Set.prototype, 1);
});

assert.throws(TypeError, function() {
  var s = new Set();
  s.has.call(Set.prototype, 1);
});
