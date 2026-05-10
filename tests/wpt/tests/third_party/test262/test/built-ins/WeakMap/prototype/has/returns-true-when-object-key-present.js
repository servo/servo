// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  Returns true when an Object key is present in the WeakMap entries list.
info: |
  WeakMap.prototype.has ( _key_ )
  5. For each Record {[[Key]], [[Value]]} _p_ of _entries_, do
    a. If _p_.[[Key]] is not ~empty~ and SameValue(_p_.[[Key]], _key_) is
      *true*, return *true*.
features: [WeakMap]
---*/

var foo = {};
var map = new WeakMap();

map.set(foo, 1);
assert.sameValue(map.has(foo), true);
