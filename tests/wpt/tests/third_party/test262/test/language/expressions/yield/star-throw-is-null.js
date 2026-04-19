// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  If iterator's "throw" method is `null`,
  IteratorClose is called before rising TypeError.
info: |
  YieldExpression : yield * AssignmentExpression

  [...]
  7. Repeat,
    [...]
    b. Else if received.[[Type]] is throw, then
      i. Let throw be ? GetMethod(iterator, "throw").
      ii. If throw is not undefined, then
        [...]
      iii. Else,
        [...]
        4. Else, perform ? IteratorClose(iteratorRecord, closeCompletion).
        [...]
        6. Throw a TypeError exception.

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.

  IteratorClose ( iteratorRecord, completion )

  [...]
  4. Let innerResult be GetMethod(iterator, "return").
  5. If innerResult.[[Type]] is normal, then
    a. Let return be innerResult.[[Value]].
    b. If return is undefined, return Completion(completion).
features: [generators, Symbol.iterator]
---*/

var throwGets = 0;
var returnGets = 0;
var iterable = {
  next: function() {
    return {value: 1, done: false};
  },
  get throw() {
    throwGets += 1;
    return null;
  },
  get return() {
    returnGets += 1;
  },
};

iterable[Symbol.iterator] = function() {
  return iterable;
};

function* generator() {
  yield* iterable;
}

var iterator = generator();
iterator.next();

assert.throws(TypeError, function() {
  iterator.throw();
});

assert.sameValue(throwGets, 1);
assert.sameValue(returnGets, 1);
