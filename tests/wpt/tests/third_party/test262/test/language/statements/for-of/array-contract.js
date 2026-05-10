// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Array entry removal during traversal using for..of
info: |
    Entries removed from an Array instance during traversal should not be
    visited.
es6id: 13.6.4
---*/

var array = [0, 1];
var iterationCount = 0;

for (var x of array) {
  assert.sameValue(x, 0);
  array.pop();
  iterationCount += 1;
}

assert.sameValue(iterationCount, 1);
