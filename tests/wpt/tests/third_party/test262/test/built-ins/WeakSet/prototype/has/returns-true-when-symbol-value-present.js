// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  Returns true when a Symbol value is present in the WeakSet entries list.
info: |
  WeakSet.prototype.has ( _value_ )
  5. For each element _e_ of _entries_, do
    a. If _e_ is not ~empty~ and SameValue(_e_, _value_) is *true*, return *true*.
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
---*/

var foo = Symbol('a description');
var s = new WeakSet();

s.add(foo);
assert.sameValue(s.has(foo), true, 'Regular symbol as value');

s.add(Symbol.hasInstance);
assert.sameValue(s.has(Symbol.hasInstance), true, 'Well-known symbol as value');
