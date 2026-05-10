// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.has
description: >
  -0 and +0 are normalized to +0;
info: |
  Map.prototype.has ( key )

  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true,
    return true.
  ...
---*/

var map = new Map();

assert.sameValue(map.has(-0), false);
assert.sameValue(map.has(+0), false);

map.set(-0, 42);
assert.sameValue(map.has(-0), true);
assert.sameValue(map.has(+0), true);

map.clear();

map.set(+0, 42);
assert.sameValue(map.has(-0), true);
assert.sameValue(map.has(+0), true);
