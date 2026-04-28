// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgenerator-prototype-throw
description: return() call while iterator is in state executing
info: |
  AsyncGenerator.prototype.return ( value )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: return, [[Value]]: value, [[Target]]: empty}.
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

var iter;
var executionorder = 0;
var valueisset = false;

async function* g() {

  iter.return(42).then(
    function(result) {
      assert(valueisset, "variable valueisset should be set to true");
      assert.sameValue(++executionorder, 2);
      assert.sameValue(result.value, 42);
      assert.sameValue(result.done, true);
    }
  ).catch($DONE);

  valueisset = true;

  yield 1;
  throw new Test262Error("This line should no be reached: generator closed by return");
}

iter = g();

iter.next().then(function(result) {

  assert.sameValue(++executionorder, 1);
  assert.sameValue(result.value, 1);
  assert.sameValue(result.done, false);

  iter.next().then(function(result) {
    assert.sameValue(++executionorder, 3);
    assert.sameValue(result.value, undefined);
    assert.sameValue(result.done, true);
  }).then($DONE, $DONE);

}).catch($DONE);
