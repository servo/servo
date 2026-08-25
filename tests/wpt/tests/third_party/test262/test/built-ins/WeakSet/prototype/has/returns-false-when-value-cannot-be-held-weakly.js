// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  Returns false if value cannot be held weakly.
info: |
  WeakSet.prototype.has ( _value_ )
  4. If CanBeHeldWeakly(_value_) is *false*, return *false*.
features: [Symbol, WeakSet]
---*/

var s = new WeakSet();

assert.sameValue(s.has(1), false);
assert.sameValue(s.has(''), false);
assert.sameValue(s.has(null), false);
assert.sameValue(s.has(undefined), false);
assert.sameValue(s.has(true), false);
assert.sameValue(s.has(Symbol.for('registered symbol')), false, 'Registered symbol not allowed as value');
