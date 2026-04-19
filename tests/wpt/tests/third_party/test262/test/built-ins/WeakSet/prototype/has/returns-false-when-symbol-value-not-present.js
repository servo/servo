// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  Return false when a Symbol value is not present in the WeakSet entries.
info: |
  WeakSet.prototype.has ( _value_ )
  6. Return *false*.
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var bar = Symbol('a description');
var s = new WeakSet();

assert.sameValue(s.has(foo), false, 'WeakSet is initially empty of regular symbol');
assert.sameValue(s.has(Symbol.hasInstance), false, 'WeakSet is initially empty of well-known symbol');

s.add(foo);
assert.sameValue(s.has(bar), false, 'Symbols with the same description are not aliased to each other');

s.delete(foo);
assert.sameValue(s.has(foo), false, 'WeakSet is again empty of regular symbol after deleting');
s.delete(Symbol.hasInstance);
assert.sameValue(s.has(Symbol.hasInstance), false, 'WeakSet is again empty of well-known symbol after deleting');
