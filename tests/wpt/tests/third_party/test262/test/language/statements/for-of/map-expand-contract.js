// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Map entry insertion during traversal using for..of
info: |
    New entries inserted into a Map instance during traversal should not be
    visited if they are removed prior to visitation.
es6id: 13.6.4
features: [Map]
---*/

var map = new Map();
var iterationCount = 0;

map.set(0, 'a');

for (var x of map) {
  assert.sameValue(x[0], 0);
  assert.sameValue(x[1], 'a');

  map.set(1, 'b');
  map.delete(1);

  iterationCount += 1;
}

assert.sameValue(iterationCount, 1);
