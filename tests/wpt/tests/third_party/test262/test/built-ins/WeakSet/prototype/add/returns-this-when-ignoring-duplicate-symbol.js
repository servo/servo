// Copyright (C) 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weakset.prototype.add
description: Returns `this` when new value is duplicate.
info: |
  WeakSet.prototype.add ( value )

  1. Let S be the this value.
  ...
  5. 5. For each element e of entries, do
    a. If e is not empty and SameValue(e, value) is true, then
      i. Return S.
  ...
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
---*/

var foo = Symbol('description');
var s = new WeakSet([foo]);

assert.sameValue(s.add(foo), s, '`s.add(foo)` returns `s`');
