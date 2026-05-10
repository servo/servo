// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: >
  Return false if value cannot be held weakly.
info: |
  WeakSet.prototype.delete ( _value_ )
  3. If CanBeHeldWeakly(_value_) is *false*, return *false*.
features: [Symbol, WeakSet]
---*/

var s = new WeakSet();

assert.sameValue(s.delete(1), false);
assert.sameValue(s.delete(''), false);
assert.sameValue(s.delete(null), false);
assert.sameValue(s.delete(undefined), false);
assert.sameValue(s.delete(true), false);
assert.sameValue(s.delete(Symbol.for('registered symbol')), false, 'Registered symbol not allowed as value');
