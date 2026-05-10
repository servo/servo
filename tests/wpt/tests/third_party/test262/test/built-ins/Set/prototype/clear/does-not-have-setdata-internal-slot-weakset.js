// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.clear
description: >
    Set.prototype.clear ( )

    ...
    3. If S does not have a [[SetData]] internal slot, throw a TypeError exception.
    ...
features: [WeakSet]
---*/

assert.throws(TypeError, function() {
  Set.prototype.clear.call(new WeakSet());
});

assert.throws(TypeError, function() {
  var s = new Set();
  s.clear.call(new WeakSet());
});
