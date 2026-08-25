// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  Returns false if the key cannot be held weakly
info: |
  WeakMap.prototype.has ( _key_ )
  4. If CanBeHeldWeakly(_key_) is *false*, return *false*.
features: [Symbol, WeakMap]
---*/

var map = new WeakMap();

assert.sameValue(map.has(1), false);
assert.sameValue(map.has(''), false);
assert.sameValue(map.has(null), false);
assert.sameValue(map.has(undefined), false);
assert.sameValue(map.has(true), false);
assert.sameValue(map.has(Symbol.for('registered symbol')), false, 'Registered symbol not allowed as key');
