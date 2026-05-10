// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  TypeError thrown when iterator `next` method returns a non-object value
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).
  4. Let received be NormalCompletion(undefined).
  5. Repeat
     a. If received.[[Type]] is normal, then
        i. Let innerResult be ? IteratorNext(iterator, received.[[Value]]).

  7.4.2 IteratorNext

  1. If value was not passed, then
     [...]
  2. Else,
     a. Let result be ? Invoke(iterator, "next", « value »).
features: [generators, Symbol.iterator]
---*/

var badIter = {};
badIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return 8;
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

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
