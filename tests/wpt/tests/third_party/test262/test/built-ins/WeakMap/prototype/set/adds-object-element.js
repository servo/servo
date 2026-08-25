// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.set
description: >
  Adds a value with an Object key.
info: |
  WeakMap.prototype.set ( _key_, _value_ )
  6. Let _p_ be the Record {[[Key]]: _key_, [[Value]]: _value_}.
  7. Append _p_ as the last element of _entries_.
features: [WeakMap]
---*/

var map = new WeakMap();
var foo = {};
var bar = {};
var baz = {};

map.set(foo, 1);
map.set(bar, 2);
map.set(baz, 3);

assert(map.has(foo));
assert(map.has(bar));
assert(map.has(baz));
