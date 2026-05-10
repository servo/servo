// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: >
  Return false if the key cannot be held weakly.
info: |
  WeakMap.prototype.delete ( _key_ )
  5. If CanBeHeldWeakly(_key_) is *false*, return *false*.
features: [Symbol, WeakMap]
---*/

var map = new WeakMap();

assert.sameValue(map.delete(1), false);
assert.sameValue(map.delete(''), false);
assert.sameValue(map.delete(NaN), false);
assert.sameValue(map.delete(null), false);
assert.sameValue(map.delete(undefined), false);
assert.sameValue(map.delete(true), false);
assert.sameValue(map.delete(Symbol.for('registered symbol')), false, 'registered symbol');
