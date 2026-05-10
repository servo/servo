// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Array entry removal and re-insertion during traversal using for..of
info: |
    Entries removed from an Array instance during traversal should be visited
    if they are re-inserted prior to iterator exhaustion.
es6id: 13.6.4
---*/

var array = [0, 1];
var iterationCount = 0;

var first = 0;
var second = 1;

for (var x of array) {
  assert.sameValue(x, first);

  first = second;
  second = null;

  if (first !== null) {
    array.pop();
    array.push(1);
  }

  iterationCount += 1;
}

assert.sameValue(iterationCount, 2);
