// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getOrInsert
description: |
  Throws TypeError if key cannot be held weakly.
info: |
  WeakMap.prototype.getOrInsert ( key, value )

  ...
  3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
  ...
features: [Symbol, WeakMap, upsert]
---*/
var s = new WeakMap();

assert.throws(TypeError, function() {
  s.getOrInsert(1, 1);
});

assert.throws(TypeError, function() {
  s.getOrInsert(false, 1);
});

assert.throws(TypeError, function() {
  s.getOrInsert(undefined, 1);
});

assert.throws(TypeError, function() {
  s.getOrInsert('string', 1);
});

assert.throws(TypeError, function() {
  s.getOrInsert(null, 1);
});

assert.throws(TypeError, function() {
  s.getOrInsert(Symbol.for('registered symbol'), 1);
}, 'Registered symbol not allowed as WeakMap key');

