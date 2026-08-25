// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
    Abrupt completion when calling IteratorValue is propagated when received.[[Type]] is return.
info: |
    14.4.14 Runtime Semantics: Evaluation
    YieldExpression : yield* AssignmentExpression

    ...
    7. Repeat,
      ...
      c. Else,
        i. Assert: received.[[Type]] is return.
        ...
        ix. If generatorKind is async, then set received to AsyncGeneratorYield(? IteratorValue(innerReturnResult)).
        ...

flags: [async]
features: [async-iteration]
---*/

var token = {};

var asyncIter = {
  [Symbol.asyncIterator]() {
      return this;
  },
  next() {
    return {
      done: false,
      value: undefined,
    };
  },
  return() {
    return {
      done: false,
      get value() {
        throw token;
      }
    };
  }
};

async function* f() {
  var thrown;
  try {
    yield* asyncIter;
  } catch (e) {
    thrown = e;
  }
  return thrown;
}

var iter = f();

iter.next().then(() => {
  iter.return().then(({value}) => {
    assert.sameValue(value, token);
  }).then($DONE, $DONE);
}).catch($DONE);
