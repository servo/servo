// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Returns undefined when an Object key is not on the WeakMap object.
info: |
  WeakMap.prototype.get ( _key_ )
  3. Let _entries_ be the List that is _M_.[[WeakMapData]].
  4. If CanBeHeldWeakly(_key_) is *false*, return *undefined*.
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not empty and SameValue(_p_.[[Key]], _key_) is *true*,
      return _p_.[[Value]].
  6. Return *undefined*.
features: [WeakMap]
---*/

var map = new WeakMap();
var key = {};

assert.sameValue(
  map.get(key), undefined,
  'returns undefined if key is not on the weakmap'
);

map.set(key, 1);
map.set({}, 2);
map.delete(key);
map.set({}, 3);

assert.sameValue(
  map.get(key), undefined,
  'returns undefined if key was deleted'
);
