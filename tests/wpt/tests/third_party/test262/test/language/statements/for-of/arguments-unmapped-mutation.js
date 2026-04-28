// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4
description: Unmapped arguments object mutation during traversal using for..of
info: |
    "Unmapped" arguments objects should be able to be traversed using a
    `for..of` loop, and dynamic changes to their contents should be reflected
    in the iterated values.
flags: [noStrict]
---*/

var expected = [1, 4, 6];
var i = 0;

(function() {
  'use strict';
  for (var value of arguments) {
    assert.sameValue(value, expected[i], 'argument at index ' + i);
    i++;
    arguments[i] *= 2;
  }
}(1, 2, 3));

assert.sameValue(i, 3, 'Visits all arguments');
