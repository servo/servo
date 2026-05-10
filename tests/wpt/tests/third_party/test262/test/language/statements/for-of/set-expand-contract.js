// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Set entry insertion and removal during traversal using for..of
info: |
    New entries inserted into a Set instance during traversal should not be
    visited if they are removed prior to visitation.
es6id: 13.6.4
features: [Set]
---*/

var set = new Set();
var iterationCount = 0;

set.add(0);

for (var x of set) {
  assert.sameValue(x, 0);

  set.add(1);
  set.delete(1);

  iterationCount += 1;
}

assert.sameValue(iterationCount, 1);
