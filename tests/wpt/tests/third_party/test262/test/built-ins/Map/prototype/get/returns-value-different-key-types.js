// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.get
description: >
  Returns the value from the specified key on different types.
info: |
  Map.prototype.get ( key )

  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true,
    return p.[[value]].
  ...
features: [Symbol]
---*/

var map = new Map();

map.set('bar', 0);
assert.sameValue(map.get('bar'), 0);

map.set(1, 42);
assert.sameValue(map.get(1), 42);

map.set(NaN, 1);
assert.sameValue(map.get(NaN), 1);

var item = {};
map.set(item, 2);
assert.sameValue(map.get(item), 2);

item = [];
map.set(item, 3);
assert.sameValue(map.get(item), 3);

item = Symbol('item');
map.set(item, 4);
assert.sameValue(map.get(item), 4);

item = null;
map.set(item, 5);
assert.sameValue(map.get(item), 5);

item = undefined;
map.set(item, 6);
assert.sameValue(map.get(item), 6);
