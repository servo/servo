// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: >
  Delete an entry with an Object key.
info: |
  WeakMap.prototype.delete ( _key_ )
  3. Let _entries_ be the List that is _M_.[[WeakMapData]].
  4. If CanBeHeldWeakly(_key_) is *false*, return *false*.
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not ~empty~ and SameValue(_p_.[[Key]], _key_) is
      *true*, then
      i. Set _p_.[[Key]] to ~empty~.
      ii. Set _p_.[[Value]] to ~empty~.
      iii. Return *true*.
features: [WeakMap]
---*/

var foo = {};
var map = new WeakMap();

map.set(foo, 42);

var result = map.delete(foo);

assert.sameValue(map.has(foo), false);
assert.sameValue(result, true, 'WeakMap#delete returns true');
