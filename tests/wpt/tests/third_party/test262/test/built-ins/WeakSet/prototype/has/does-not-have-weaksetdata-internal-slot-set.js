// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  Throws TypeError if context doesn't have a [[WeakSetData]] internal slot.
info: |
  WeakSet.prototype.has ( value )

  ...
  3. If S does not have a [[WeakSetData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

assert.throws(TypeError, function() {
  WeakSet.prototype.has.call(new Set(), {});
});

assert.throws(TypeError, function() {
  var s = new WeakSet();
  s.has.call(new Set(), {});
});
