// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getOrInsert
description: |
  Returns the value from the specified Symbol key
info: |
  WeakMap.prototype.getOrInsert ( key, value)

  ...
  4. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  ...
features: [Symbol, WeakMap, symbols-as-weakmap-keys, upsert]
---*/
var foo = Symbol('a description');
var bar = Symbol('a description');
var baz = Symbol('different description');
var map = new WeakMap([
  [foo, 0],
]);

assert.sameValue(map.getOrInsert(foo), 0, 'Regular symbol as key, added in constructor');

map.set(bar, 1);
map.set(baz, 2);
assert.sameValue(map.getOrInsert(baz), 2, 'Regular symbol as key, added with set()');
assert.sameValue(map.getOrInsert(bar), 1, "Symbols with the same description don't overwrite each other");

map.set(Symbol.hasInstance, 3);
assert.sameValue(map.getOrInsert(Symbol.hasInstance), 3, 'Well-known symbol as key');

