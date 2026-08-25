// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: Value received from invocation of generator's `next` method
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
        iii. If done is true, then
             1. Return ? IteratorValue(innerResult).

  7.4.4 IteratorValue

  1. Assert: Type(iterResult) is Object.
  2. Return ? Get(iterResult, "value").
features: [generators, Symbol.iterator]
---*/

var quickIter = {};
var exprValue, nextReceived, done, iter;
quickIter[Symbol.iterator] = function() {
  return {
    next: function(x) {
      nextReceived = x;
      return {
        done: done,
        value: 3333
      };
    }
  };
};
function* g() {
  exprValue = yield * quickIter;
}

done = true;
iter = g();
iter.next(4444);

assert.sameValue(
  nextReceived, undefined, 'received value (previously-exhausted iterator)'
);
assert.sameValue(exprValue, 3333, 'expression value (previously-exhausted iterator)');

done = false;
exprValue = null;
iter = g();
iter.next(2222);
done = true;
iter.next(5555);

assert.sameValue(nextReceived, 5555, 'received value');
assert.sameValue(exprValue, 3333, 'expression value');
