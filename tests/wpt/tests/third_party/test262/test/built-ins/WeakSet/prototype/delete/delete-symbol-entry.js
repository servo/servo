// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: >
  Delete an entry that is a Symbol
info: |
  WeakSet.prototype.delete ( _value_ )
  4. Let _entries_ be the List that is _S_.[[WeakSetData]].
  5. For each element _e_ of _entries_, do
    a. If _e_ is not ~empty~ and SameValue(_e_, _value_) is *true*, then
      i. Replace the element of _entries_ whose value is _e_ with an element
        whose value is ~empty~.
      ii. Return *true*.
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var bar = Symbol('a description');
var s = new WeakSet();

s.add(foo);
s.add(bar);
s.add(Symbol.hasInstance);

assert.sameValue(s.delete(foo), true, 'Returns true for regular symbol');
assert(!s.has(foo), 'Regular symbol is removed from set');
assert(s.has(bar), 'Symbols with the same description are not aliased to each other');

assert.sameValue(s.delete(Symbol.hasInstance), true, 'Returns true for well-known symbol');
assert(!s.has(Symbol.hasInstance), 'Well-known symbol is removed from set');
