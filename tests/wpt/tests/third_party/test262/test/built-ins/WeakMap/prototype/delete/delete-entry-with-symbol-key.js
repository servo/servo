// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: >
  Delete an entry with a Symbol key.
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
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var bar = Symbol('a description');
var map = new WeakMap();

map.set(foo, 42);
map.set(bar, 43);

var result = map.delete(foo);

assert(!map.has(foo), 'Regular symbol was deleted from map');
assert(map.has(bar), "Symbols with the same description don't alias each other");
assert.sameValue(result, true, 'delete() returns true for regular symbol');

map.set(Symbol.hasInstance, 44);

result = map.delete(Symbol.hasInstance);

assert(!map.has(Symbol.hasInstance), 'Well-known symbol was deleted from map');
assert.sameValue(result, true, 'delete() returns true for well-known symbol');
