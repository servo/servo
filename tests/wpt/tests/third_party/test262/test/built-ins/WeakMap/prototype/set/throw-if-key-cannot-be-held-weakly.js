// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: Throws TypeError if key cannot be held weakly.
info: |
  WeakMap.prototype.set ( _key_, _value_ )
  4. If CanBeHeldWeakly(_key_) is *false*, throw a *TypeError* exception.
features: [Symbol, WeakMap]
---*/

var s = new WeakMap();

assert.throws(TypeError, function() {
  s.set(1, 1);
});

assert.throws(TypeError, function() {
  s.set(false, 1);
});

assert.throws(TypeError, function() {
  s.set(undefined, 1);
});

assert.throws(TypeError, function() {
  s.set('string', 1);
});

assert.throws(TypeError, function() {
  s.set(null, 1);
});

assert.throws(TypeError, function() {
  s.set(Symbol.for('registered symbol'), 1);
}, 'Registered symbol not allowed as WeakMap key');
