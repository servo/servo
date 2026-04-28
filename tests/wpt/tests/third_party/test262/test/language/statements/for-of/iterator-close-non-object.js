// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    If an iterator's `return` method returns a non-Object value, a TypeError
    should be thrown.
features: [Symbol.iterator]
---*/

var iterable = {};
var iterationCount = 0;

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false, value: null };
    },
    return: function() {
      return 0;
    }
  };
};

assert.throws(TypeError, function() {
  for (var x of iterable) {
    iterationCount += 1;
    break;
  }
});

assert.sameValue(iterationCount, 1, 'The loop body is evaluated');
