// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.add
description: >
  Throws TypeError if context doesn't have a [[WeakSetData]] internal slot.
info: |
  WeakSet.prototype.add ( value )

  ...
  3. If S does not have a [[WeakSetData]] internal slot, throw a TypeError
  exception.
  ...

---*/

assert.throws(TypeError, function() {
  WeakSet.prototype.add.call([], {});
});

assert.throws(TypeError, function() {
  var s = new WeakSet();
  s.add.call([], {});
});
