// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String traversal using for..of
info: |
    String literals should be able to be traversed using a `for...of` loop. The
    loop body should execute once for every BMP character.
es6id: 13.6.4
---*/

var string = 'abc';
var first = 'a';
var second = 'b';
var third = 'c';

var iterationCount = 0;

for (var value of string) {
  assert.sameValue(value, first);
  first = second;
  second = third;
  third = null;
  iterationCount += 1;
}

assert.sameValue(iterationCount, 3);
