// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  Return false when a Symbol key is not present in the WeakMap entries.
info: |
  WeakMap.prototype.has ( _key_ )
  6. Return *false*.
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var bar = Symbol('a description');
var map = new WeakMap();

assert.sameValue(map.has(foo), false, 'WeakMap is initially empty of regular symbol');
assert.sameValue(map.has(Symbol.hasInstance), false, 'WeakMap is initially empty of well-known symbol');

map.set(foo, 1);
assert.sameValue(map.has(bar), false, "Symbols with the same description don't alias each other");

map.delete(foo);
assert.sameValue(map.has(foo), false, 'WeakMap is empty again of regular symbol after deleting');

map.set(Symbol.hasInstance, 2);
map.delete(Symbol.hasInstance);
assert.sameValue(map.has(Symbol.hasInstance), false, 'WeakMap is empty again of well-known symbol after deleting');
