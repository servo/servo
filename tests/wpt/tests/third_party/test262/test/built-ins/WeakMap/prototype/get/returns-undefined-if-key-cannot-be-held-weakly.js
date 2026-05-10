// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Returns undefined when key cannot be held weakly.
info: |
  WeakMap.prototype.get ( _key_ )
  4. If CanBeHeldWeakly(_key_) is *false*, return *undefined*.
features: [Symbol, WeakMap]
---*/

var map = new WeakMap();

assert.sameValue(map.get(null), undefined, 'Returns undefined if key is null');

assert.sameValue(map.get(NaN), undefined, 'Returns undefined if key is NaN');

assert.sameValue(
  map.get('foo'), undefined,
  'Returns undefined if key is a String'
);

assert.sameValue(
  map.get(1), undefined,
  'Returns undefined if key is a Number'
);

assert.sameValue(
  map.get(undefined), undefined,
  'Returns undefined if key is undefined'
);

assert.sameValue(
  map.get(Symbol.for('registered symbol')), undefined,
  'Returns undefined if key is a registered Symbol'
);
