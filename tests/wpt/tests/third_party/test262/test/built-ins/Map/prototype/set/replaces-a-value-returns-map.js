// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Map.prototype.set returns the given `this` map object.
info: |
  Map.prototype.set ( key , value )

  1. Let M be the this value.
  ...
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true, then
      i. Set p.[[value]] to value.
      ii. Return M.
  ...
---*/

var map = new Map([['item', 0]]);
var map2 = new Map();

var x = map.set('item', 42);
assert.sameValue(x, map);

x = Map.prototype.set.call(map, 'item', 0);
assert.sameValue(x, map);

x = map2.set.call(map, 'item', 0);
assert.sameValue(x, map, 'Map#set returns the map `this` value');
