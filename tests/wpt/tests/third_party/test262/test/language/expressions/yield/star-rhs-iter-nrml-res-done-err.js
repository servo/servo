// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  Abrupt completion returned when accessing `done` property of iteration result
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).
  4. Let received be NormalCompletion(undefined).
  5. Repeat
     a. If received.[[Type]] is normal, then
        i. Let innerResult be ? IteratorNext(iterator, received.[[Value]]).
        ii. Let done be ? IteratorComplete(innerResult).

  7.4.3 IteratorComplete

  1. Assert: Type(iterResult) is Object.
  2. Return ToBoolean(? Get(iterResult, "done")).
features: [generators, Symbol.iterator]
---*/

var thrown = new Test262Error();
var badIter = {};
var poisonedDone = Object.defineProperty({}, 'done', {
  get: function() {
    throw thrown;
  }
});
badIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedDone;
    }
  };
};
function* g() {
  try {
    yield * badIter;
  } catch (err) {
    caught = err;
  }
}
var iter = g();
var result, caught;

result = iter.next();

assert.sameValue(caught, thrown);
