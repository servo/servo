// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Array entry insertion during traversal using for..of
info: |
    New entries inserted into an Array instance during traversal should be
    visited.
es6id: 13.6.4
---*/

var array = [0];
var iterationCount = 0;

var first = 0;
var second = 1;

for (var x of array) {
  assert.sameValue(x, first);

  first = second;
  second = null;

  if (first !== null) {
    array.push(1);
  }

  iterationCount += 1;
}

assert.sameValue(iterationCount, 2);
