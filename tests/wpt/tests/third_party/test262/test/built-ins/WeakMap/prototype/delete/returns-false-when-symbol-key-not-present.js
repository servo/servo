// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: >
  Return false if a Symbol key is not in the WeakMap.
info: |
  WeakMap.prototype.delete ( _key_ )
  6. Return *false*.
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var map = new WeakMap();
var foo = Symbol('a description');
var bar = Symbol('a description');
var baz = Symbol('another description');

map.set(foo, 42);

assert.sameValue(map.delete(baz), false, 'Regular symbol key not present')
assert.sameValue(map.delete(bar), false, "Symbols with the same description don't alias to each other");
assert.sameValue(map.delete(Symbol.hasInstance), false, 'Well-known symbol key not present');
