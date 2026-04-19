// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Input throw-completion forwarded when IteratorClose returns abruptly because GetMethod throws.
info: |
  27.2.4.3 Promise.any ( iterable )
    ...
    7. Let result be Completion(PerformPromiseAny(iteratorRecord, C, promiseCapability, promiseResolve)).
    8. If result is an abrupt completion, then
      a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
      b. IfAbruptRejectPromise(result, promiseCapability).
    ...

  7.4.11 IteratorClose ( iteratorRecord, completion )
    ...
    3. Let innerResult be Completion(GetMethod(iterator, "return")).
    ...
    5. If completion is a throw completion, return ? completion.
    ...

  7.3.10 GetMethod ( V, P )
    1. Let func be ? GetV(V, P).
    2. If func is either undefined or null, return undefined.
    3. If IsCallable(func) is false, throw a TypeError exception.
    ...
---*/

function resolve() {
  throw new Test262Error("Unexpected call to resolve");
}

var rejectCallCount = 0;

function reject(e) {
  rejectCallCount += 1;
  assert.sameValue(e, "bad promise resolve");
}

class BadPromise {
  constructor(executor) {
    executor(resolve, reject);
  }

  static resolve() {
    throw "bad promise resolve";
  }
}

for (var returnMethod of [0, 0n, true, "string", {}, Symbol()]) {
  var iterator = {
    [Symbol.iterator]() {
      return this;
    },
    next() {
      return {done: false};
    },
    return: returnMethod,
  };

  // Reset counter.
  rejectCallCount = 0;

  Promise.any.call(BadPromise, iterator);

  // Ensure `reject` was called exactly once.
  assert.sameValue(rejectCallCount, 1);
}
