// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgenerator-prototype-throw
description: return() call while iterator is in state executing
info: |
  AsyncGenerator.prototype.throw ( error )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: throw, [[Value]]: error, [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  4. Let queue be generator.[[AsyncGeneratorQueue]].
  5. Let request be AsyncGeneratorRequest{[[Completion]]: completion,
     [[Capability]]: promiseCapability}.
  6. Append request to the end of queue.
  ...

  AsyncGeneratorResolve ( generator, value, done )
  ...
  2. Let queue be generator.[[AsyncGeneratorQueue]].
  3. Assert: queue is not an empty List.
  4. Remove the first element from queue and let next be the value of that element.
  ...

flags: [async]
features: [async-iteration]
---*/

var iter, result;
var thrownErr = new Error("Catch me.");
var caughtErr;

var order = 0;
var promises = []

async function* g() {

  iter.throw(thrownErr).then(
    function() {
      $DONE(new Test262Error("throw() should result in reject promise."));
    },
    function(e) {
      caughtErr = e;
      order++;
    }
  );

  yield 1;
  yield 2;
}

iter = g();

iter.next().then(function(result) {

  assert.sameValue(++order, 1);
  assert.sameValue(result.value, 1);
  assert.sameValue(result.done, false);

  iter.next().then(function(result) {
    assert.sameValue(++order, 3);
    assert.sameValue(caughtErr, thrownErr);
    assert.sameValue(result.done, true);
  }).then($DONE, $DONE);

}).catch($DONE)
