// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-throw
description: >
  "throw" returns a rejected promise
info: |
  AsyncGenerator.prototype.next ( value )
  1. Let generator be the this value.
  2. Let completion be NormalCompletion(value).
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  4. Let queue be generator.[[AsyncGeneratorQueue]].
  5. Let request be AsyncGeneratorRequest{[[Completion]]: completion,
     [[Capability]]: promiseCapability}.
  6. Append request to the end of queue.
  ...
  9. Return promiseCapability.[[Promise]].

  AsyncGeneratorReject ( generator, exception )
  1. Assert: generator is an AsyncGenerator instance.
  2. Let queue be generator.[[AsyncGeneratorQueue]].
  3. Assert: queue is not an empty List.
  4. Remove the first element from queue and let next be the value of that element.
  5. Let promiseCapability be next.[[Capability]].
  6. Perform ! Call(promiseCapability.[[Reject]], undefined, « exception »).
  ...

flags: [async]
features: [async-iteration]
---*/

async function* g() {}

var errormessage = "Promise rejected."
var result = g().throw(new Test262Error(errormessage))

assert(result instanceof Promise, "Expected result to be an instanceof Promise")

result.then(
  function () {
    throw new Test262Error("Expected result to be rejected promise.");
  },
  function (e) {
    if (!(e.message = errormessage)) {
      throw new Test262Error("Expected thrown custom error, got " + e);
    }
  }
).then($DONE, $DONE)
