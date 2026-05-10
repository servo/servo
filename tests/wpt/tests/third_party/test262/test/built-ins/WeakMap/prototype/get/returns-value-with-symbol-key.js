// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Returns the value from the specified Symbol key
info: |
  WeakMap.prototype.get ( _key_ )
  3. Let _entries_ be the List that is _M_.[[WeakMapData]].
  4. If CanBeHeldWeakly(_key_) is *false*, return *undefined*.
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not ~empty~ and SameValue(_p_.[[Key]], _key_) is
      *true*, return _p_.[[Value]].
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var bar = Symbol('a description');
var baz = Symbol('different description');
var map = new WeakMap([
  [foo, 0],
]);

assert.sameValue(map.get(foo), 0, 'Regular symbol as key, added in constructor');

map.set(bar, 1);
map.set(baz, 2);
assert.sameValue(map.get(baz), 2, 'Regular symbol as key, added with set()');
assert.sameValue(map.get(bar), 1, "Symbols with the same description don't overwrite each other");

map.set(Symbol.hasInstance, 3);
assert.sameValue(map.get(Symbol.hasInstance), 3, 'Well-known symbol as key');
