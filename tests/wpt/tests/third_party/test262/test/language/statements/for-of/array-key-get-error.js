// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Error in Array entry access during traversal using for..of
info: |
    If retrieving an element from the array produces an error, that error
    should be forwarded to the run time.
es6id: 13.6.4
---*/

var array = [];
var iterationCount = 0;

Object.defineProperty(array, '0', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  for (var value of array) {
    iterationCount += 1;
  }
});

assert.sameValue(iterationCount, 0, 'The loop body is not evaluated');
