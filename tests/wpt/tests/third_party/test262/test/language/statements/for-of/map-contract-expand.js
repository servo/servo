// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Map entry removal and re-insertion during traversal using for..of
info: |
    Entries removed from a Map instance during traversal should be visited if
    they are re-inserted prior to iterator exhaustion.
es6id: 13.6.4
features: [Map]
---*/

var map = new Map();
var iterationCount = 0;

var first = [0, 'a'];
var second = [1, 'b'];

map.set(0, 'a');
map.set(1, 'b');

for (var x of map) {
  assert.sameValue(x[0], first[0]);
  assert.sameValue(x[1], first[1]);

  first = second;
  second = null;

  if (first !== null) {
    map.delete(1);
    map.set(1, 'b');
  }

  iterationCount += 1;
}

assert.sameValue(iterationCount, 2);
