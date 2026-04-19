// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.2.2
description: >
  Already resolved promise is not rejected when then() function throws an exception.
info: |
  PromiseResolveThenableJob ( promiseToResolve, thenable, then )

  1. Let resolvingFunctions be CreateResolvingFunctions(promiseToResolve).
  2. Let thenCallResult be Call(then, thenable, «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).
  3. If thenCallResult is an abrupt completion,
    a. Let status be Call(resolvingFunctions.[[Reject]], undefined, «thenCallResult.[[value]]»)
    b. NextJob Completion(status).
  ...
flags: [async]
---*/

var thenable = {
  then: function(resolve) {
    resolve();
  }
};

var thenableWithError = {
  then: function(resolve) {
    resolve(thenable);
    throw new Error("ignored exception");
  }
};

function executor(resolve, reject) {
  resolve(thenableWithError);
}

new Promise(executor).then($DONE, $DONE);
