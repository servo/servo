// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-return
description: >
 "return" returns a promise for an IteratorResult object
info: |
  AsyncGenerator.prototype.return ( value )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: return, [[Value]]: value,
     [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  4. Let queue be generator.[[AsyncGeneratorQueue]].
  5. Let request be AsyncGeneratorRequest{[[Completion]]: completion,
     [[Capability]]: promiseCapability}.
  6. Append request to the end of queue.
  ...

  AsyncGeneratorResolve ( generator, value, done )
  1. Assert: generator is an AsyncGenerator instance.
  2. Let queue be generator.[[AsyncGeneratorQueue]].
  3. Assert: queue is not an empty List.
  4. Remove the first element from queue and let next be the value of that element.
  5. Let promiseCapability be next.[[Capability]].
  6. Let iteratorResult be ! CreateIterResultObject(value, done).
  7. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iteratorResult »).
  ...

flags: [async]
features: [async-iteration]
---*/

async function* g() {}

g().return().then(function (result) {
  assert(
    Object.prototype.hasOwnProperty.call(result, 'value'), 'Has "own" property `value`'
  );
  assert(
    Object.prototype.hasOwnProperty.call(result, 'done'), 'Has "own" property `done`'
  );
  assert.sameValue(Object.getPrototypeOf(result), Object.prototype);
}).then($DONE, $DONE)
