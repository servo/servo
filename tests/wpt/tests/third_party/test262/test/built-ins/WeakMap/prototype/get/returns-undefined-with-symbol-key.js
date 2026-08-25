// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Returns undefined when a Symbol key is not on the WeakMap object.
info: |
  WeakMap.prototype.get ( _key_ )
  3. Let _entries_ be the List that is _M_.[[WeakMapData]].
  4. If CanBeHeldWeakly(_key_) is *false*, return *undefined*.
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not empty and SameValue(_p_.[[Key]], _key_) is *true*,
      return _p_.[[Value]].
  6. Return *undefined*.
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var map = new WeakMap();
var key = Symbol('a description');

assert.sameValue(map.get(key), undefined, 'returns undefined for regular symbol on initially empty map');
assert.sameValue(
  map.get(Symbol.hasInstance),
  undefined,
  'returns undefined for well-known symbol on initially empty map'
);

map.set(key, 1);
map.set(Symbol.hasInstance, 2);
map.delete(key);
map.delete(Symbol.hasInstance);

assert.sameValue(map.get(key), undefined, 'returns undefined for deleted regular symbol');
assert.sameValue(map.get(Symbol.hasInstance), undefined, 'returns undefined for deleted well-known symbol');
