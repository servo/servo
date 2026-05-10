// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.3.1
description: >
  Already resolved promise is not rejected when executor throws an exception.
info: |
  Promise ( executor )

  ...
  8. Let resolvingFunctions be CreateResolvingFunctions(promise).
  9. Let completion be Call(executor, undefined, «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).
  10. If completion is an abrupt completion, then
    a. Let status be Call(resolvingFunctions.[[Reject]], undefined, «completion.[[value]]»).
    b. ReturnIfAbrupt(status).
  ...
flags: [async]
---*/

var thenable = {
  then: function(resolve) {
    resolve();
  }
};

function executor(resolve, reject) {
  resolve(thenable);
  throw new Error("ignored exception");
}

new Promise(executor).then($DONE, $DONE);
