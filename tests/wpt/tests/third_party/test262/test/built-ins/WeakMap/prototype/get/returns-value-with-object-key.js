// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Returns the value from the specified Object key
info: |
  WeakMap.prototype.get ( _key_ )
  3. Let _entries_ be the List that is _M_.[[WeakMapData]].
  4. If CanBeHeldWeakly(_key_) is *false*, return *undefined*.
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not ~empty~ and SameValue(_p_.[[Key]], _key_) is
      *true*, return _p_.[[Value]].
features: [WeakMap]
---*/

var foo = {};
var bar = {};
var baz = [];
var map = new WeakMap([
  [foo, 0]
]);

assert.sameValue(map.get(foo), 0);

map.set(bar, 1);
assert.sameValue(map.get(bar), 1);

map.set(baz, 2);
assert.sameValue(map.get(baz), 2);
