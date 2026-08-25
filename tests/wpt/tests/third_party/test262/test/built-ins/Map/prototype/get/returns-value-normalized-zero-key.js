// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.get
description: >
  -0 and +0 are normalized to +0;
info: |
  Map.prototype.get ( key )

  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true,
    return p.[[value]].
  ...
---*/

var map = new Map();

map.set(+0, 42);
assert.sameValue(map.get(-0), 42);

map = new Map();
map.set(-0, 43);
assert.sameValue(map.get(+0), 43);
