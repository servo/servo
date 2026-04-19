// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String traversal using for..of (incomplete surrogate pairs)
info: |
    String literals should be able to be traversed using a `for...of` loop. The
    loop body should execute once for each incomplete surrogate pair.
es6id: 13.6.4
---*/

var string = 'a\ud801b\ud801';
var first = 'a';
var second = '\ud801';
var third = 'b';
var fourth = '\ud801';

var iterationCount = 0;

for (var value of string) {
  assert.sameValue(value, first);
  first = second;
  second = third;
  third = fourth;
  fourth = null;
  iterationCount += 1;
}

assert.sameValue(iterationCount, 4);
