// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: Adds a value with a Symbol key.
info: |
  WeakMap.prototype.set ( _key_, _value_ )
  6. Let _p_ be the Record {[[Key]]: _key_, [[Value]]: _value_}.
  7. Append _p_ as the last element of _entries_.
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var map = new WeakMap();
var foo = Symbol('a description');
var bar = Symbol('a description');

map.set(foo, 1);
map.set(bar, 2);
map.set(Symbol.hasInstance, 3);

assert(map.has(bar), 'Regular symbol as key');
assert.sameValue(map.get(foo), 1, "Symbols with the same description don't overwrite each other");
assert(map.has(Symbol.hasInstance), 'Well-known symbol as key');
