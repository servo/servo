// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: Invocation of iterator `throw` method
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
            1. Let innerResult be ? Call(throw, iterator, « received.[[Value]]
               »).
            [...]
features: [generators, Symbol.iterator]
---*/

var args, thisValue;
var callCount = 0;
var spyIterator = {
  next: function() {
    return { done: false };
  },
  throw: function() {
    callCount += 1;
    args = arguments;
    thisValue = this;
    return { done: true };
  }
};
var spyIterable = {};
spyIterable[Symbol.iterator] = function() {
  return spyIterator;
};
function* g() {
  yield * spyIterable;
}
var iter = g();

iter.next(8888);
iter.throw(7777);

assert.sameValue(callCount, 1);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], 7777);
assert.sameValue(thisValue, spyIterator);
