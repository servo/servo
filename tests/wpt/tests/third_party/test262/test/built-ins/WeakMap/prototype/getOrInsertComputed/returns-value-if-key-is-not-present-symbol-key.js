// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  Returns the value from the specified Symbol key
info: |
  WeakMap.prototype.getOrInsertComputed ( key, callbackfn )

  ...
  8. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  9. Append p to M.[[WeakMapData]].
  10. Return value.
features: [Symbol, WeakMap, symbols-as-weakmap-keys, upsert]
---*/
var foo = Symbol('a description');
var bar = Symbol('a description');
var baz = Symbol('different description');
var map = new WeakMap();

assert.sameValue(map.getOrInsertComputed(foo, () => 0), 0, 'Regular symbol as key, added in constructor');
assert.sameValue(map.getOrInsertComputed(baz, () => 2), 2, 'Regular symbol as key, added with set()');
assert.sameValue(map.getOrInsertComputed(bar, () => 1), 1, "Symbols with the same description don't overwrite each other");
assert.sameValue(map.getOrInsertComputed(Symbol.hasInstance, () => 3), 3, 'Well-known symbol as key');

