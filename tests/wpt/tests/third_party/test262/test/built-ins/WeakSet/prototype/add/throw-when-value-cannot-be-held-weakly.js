// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.add
description: Throws TypeError if value cannot be held weakly.
info: |
  WeakSet.prototype.add ( _value_ )
  3. If CanBeHeldWeakly(_value_) is *false*, throw a *TypeError* exception.
features: [Symbol, WeakSet]
---*/

var s = new WeakSet();

assert.throws(TypeError, function() {
  s.add(1);
});

assert.throws(TypeError, function() {
  s.add(false);
});

assert.throws(TypeError, function() {
  s.add();
});

assert.throws(TypeError, function() {
  s.add('string');
});

assert.throws(TypeError, function() {
  s.add(null);
});

assert.throws(TypeError, function() {
  s.add(Symbol.for('registered symbol'));
}, 'Registered symbol not allowed as WeakSet value');
