// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  Non-object value returned by iterator `return` method following protocol
  violation
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).
  4. Let received be NormalCompletion(undefined).
  5. Repeat
     a. If received.[[Type]] is normal, then
        [...]
     b. Else if received.[[Type]] is throw, then
        i. Let throw be ? GetMethod(iterator, "throw").
        ii. If throw is not undefined, then
            [...]
        iii. Else,
             1. NOTE: If iterator does not have a throw method, this throw is
                going to terminate the yield* loop. But first we need to give
                iterator a chance to clean up.
             2. Perform ? IteratorClose(iterator, Completion{[[Type]]: normal,
                [[Value]]: empty, [[Target]]: empty}).

  7.4.6 IteratorClose

  1. Assert: Type(iterator) is Object.
  2. Assert: completion is a Completion Record.
  3. Let return be ? GetMethod(iterator, "return").
  4. If return is undefined, return Completion(completion).
  5. Let innerResult be Call(return, iterator, « »).
  6. If completion.[[Type]] is throw, return Completion(completion).
  7. If innerResult.[[Type]] is throw, return Completion(innerResult).
  8. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
features: [generators, Symbol.iterator]
---*/

var badIter = {};
badIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false };
    },
    return: function() {
      return 87;
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
var caught;

iter.next();
iter.throw();

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
