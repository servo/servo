// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.g
description: >
    If `nextResult` is an abrupt completion as per IteratorStep (ES6 7.4.5),
    return the completion.
features: [generators]
---*/

var iterable = (function*() {
  throw new Test262Error();
}());
var iterationCount = 0;

assert.throws(Test262Error, function() {
  for (var x of iterable) {
    iterationCount += 1;
  }
});

assert.sameValue(iterationCount, 0, 'The loop body is not evaluated');
