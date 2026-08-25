// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.5.3
description: >
  Promise reaction jobs do not check for cyclic resolutions.
info: |
  Promise.prototype.then ( onFulfilled , onRejected )

  ...
  5. Let resultCapability be NewPromiseCapability(C).
  6. ReturnIfAbrupt(resultCapability).
  7. Return PerformPromiseThen(promise, onFulfilled, onRejected, resultCapability).

  25.4.5.3.1 PerformPromiseThen ( promise, onFulfilled, onRejected, resultCapability )
    ...
    3. If IsCallable(onFulfilled) is false, then
      a. Let onFulfilled be "Identity".
    4. If IsCallable(onRejected) is false, then
      a. Let onRejected be "Thrower".
    5. Let fulfillReaction be the PromiseReaction { [[Capabilities]]: resultCapability, [[Handler]]: onFulfilled }.
    6. Let rejectReaction be the PromiseReaction { [[Capabilities]]: resultCapability, [[Handler]]: onRejected}.
    ...
    8. Else if the value of promise's [[PromiseState]] internal slot is "fulfilled",
      a. Let value be the value of promise's [[PromiseResult]] internal slot.
      b. Perform EnqueueJob("PromiseJobs", PromiseReactionJob, «fulfillReaction, value»).
    ...

  25.4.2.1 PromiseReactionJob ( reaction, argument )
    ...
    4. If handler is "Identity", let handlerResult be NormalCompletion(argument).
    ...
    8. Let status be Call(promiseCapability.[[Resolve]], undefined, «handlerResult.[[value]]»).
    9. NextJob Completion(status).
features: [class]
flags: [async]
---*/

var createBadPromise = false;
var object = {};

class P extends Promise {
  constructor(executor) {
    if (createBadPromise) {
      executor(
        function(v) {
          assert.sameValue(v, object);
          $DONE();
        },
        function(e) {
          $DONE(e);
        }
      );
      return object;
    }
    return super(executor);
  }
}

var p = P.resolve(object);

createBadPromise = true;
var q = p.then();
createBadPromise = false;

assert.sameValue(q, object, "then() returns object");
