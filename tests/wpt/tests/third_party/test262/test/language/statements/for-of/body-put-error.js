// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: >
  If the left-hand side is not a lexical binding and the assignment produces
  an error, the iterator should be closed and the error forwarded to the
  runtime.
info: |
  ...
  If destructuring is false, then
    If lhsRef is an abrupt completion, then
      Let status be lhsRef.
    Else if lhsKind is lexicalBinding, then
      Let status be InitializeReferencedBinding(lhsRef, nextValue).
    Else,
      Let status be PutValue(lhsRef, nextValue).
  ...

features: [for-of, Symbol.iterator]
---*/

var callCount = 0;
var iterationCount = 0;
var iterable = {};
var x = {
  set attr(_) {
    throw new Test262Error();
  }
};

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false, value: 0 };
    },
    return: function() {
      callCount += 1;
    }
  }
};

assert.throws(Test262Error, function() {
  for (x.attr of iterable) {
    iterationCount += 1;
  }
});

assert.sameValue(iterationCount, 0, 'The loop body is not evaluated');
assert.sameValue(callCount, 1, 'Iterator is closed');
