// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Array entry insertion and removal items during traversal using for..of
info: |
    New entries inserted into an Array instance during traversal should not be
    visited if they are removed prior to visitation.
es6id: 13.6.4
---*/

var array = [0];
var iterationCount = 0;

for (var x of array) {
  assert.sameValue(x, 0);

  array.push(1);
  array.pop();

  iterationCount += 1;
}

assert.sameValue(iterationCount, 1);
