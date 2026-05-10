// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.add
description: Adds a Symbol value.
info: |
  WeakSet.prototype.add ( _value_ )
  6. Append _value_ as the last element of _entries_.
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
---*/

var s = new WeakSet();
var foo = Symbol('a description');
var bar = Symbol('a description');
var baz = Symbol('a different description');

s.add(foo);
s.add(baz);
s.add(Symbol.hasInstance);

assert(s.has(foo), 'Regular symbol');
assert(!s.has(bar), "Symbols with the same description don't alias each other");
assert(s.has(baz), 'Regular symbol with different description');
assert(s.has(Symbol.hasInstance), 'Well-known symbol');
