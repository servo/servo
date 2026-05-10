// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-destructuringassignmentevaluation
description: >
  Input throw-completion forwarded when IteratorClose returns abruptly because GetV in GetMethod throws.
info: |
  13.15.5.2 Runtime Semantics: DestructuringAssignmentEvaluation

  ArrayAssignmentPattern : [ AssignmentElementList , Elisionopt AssignmentRestElementopt ]
    ...
    2. Let status be Completion(IteratorDestructuringAssignmentEvaluation of AssignmentElementList with argument iteratorRecord).
    3. If status is an abrupt completion, then
      a. If iteratorRecord.[[Done]] is false, return ? IteratorClose(iteratorRecord, status).
      b. Return ? status.
    ...

  7.4.11 IteratorClose ( iteratorRecord, completion )
    ...
    3. Let innerResult be Completion(GetMethod(iterator, "return")).
    ...
    5. If completion is a throw completion, return ? completion.
    ...

  7.3.10 GetMethod ( V, P )
    1. Let func be ? GetV(V, P).
    ...
---*/

function MyError() {}

function thrower() {
  throw new MyError();
}

var returnGetterCalled = 0;

var iterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {done: false};
  },
  get return() {
    returnGetterCalled += 1;
    throw "bad";
  }
};

assert.throws(MyError, function() {
  var a;
  ([a = thrower()] = iterator);
});

assert.sameValue(returnGetterCalled, 1);
