// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  Abrupt completion returned after protocol violation (and a `return` method
  is defined)
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
             3. NOTE: The next step throws a TypeError to indicate that there
                was a yield* protocol violation: iterator does not have a throw
                method.
             4. Throw a TypeError exception.

  7.4.6 IteratorClose

  1. Assert: Type(iterator) is Object.
  2. Assert: completion is a Completion Record.
  3. Let return be ? GetMethod(iterator, "return").
  4. If return is undefined, return Completion(completion).
  5. Let innerResult be Call(return, iterator, « »).
  6. If completion.[[Type]] is throw, return Completion(completion).
  7. If innerResult.[[Type]] is throw, return Completion(innerResult).
  8. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
  9. Return Completion(completion).
features: [generators, Symbol.iterator]
---*/

var badIter = {};
var throwCount = 0;
var thisValue, args;
var poisonedReturn = {
  next: function() {
    return { done: false };
  },
  return: function() {
    thisValue = this;
    args = arguments;
    return 'this value has no effect on the protocol';
  }
};
Object.defineProperty(poisonedReturn, 'throw', {
  get: function() {
    throwCount += 1;
  }
});
badIter[Symbol.iterator] = function() {
  return poisonedReturn;
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

assert.sameValue(throwCount, 1);
assert.sameValue(thisValue, poisonedReturn, '"this" value');
assert.sameValue(
  typeof args, 'object', 'method invoked, arguments object provided'
);
assert.sameValue(args.length, 0);
assert.sameValue(typeof caught, 'object', 'object value thrown');
assert.sameValue(caught.constructor, TypeError);
