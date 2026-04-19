// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will call default sync throw
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let throw be GetMethod(syncIterator, "throw").
  ...
  8. Let throwResult be Call(throw, syncIterator, « value »)
  9. IfAbruptRejectPromise(throwResult, promiseCapability).
  ...
  22. Return promiseCapability.[[Promise]].

  Generator.prototype.throw ( exception )
  1. Let g be the this value.
  2. Let C be Completion{[[Type]]: throw, [[Value]]: exception, [[Target]]: empty}.
  3. Return ? GeneratorResumeAbrupt(g, C).

flags: [async]
features: [async-iteration]
---*/

var thrownError = new Error("Catch me.")

function* g() {
  yield 42;
  throw new Test262Error('throw closes iter');
  yield 43;
}

async function* asyncg() {
  yield* g();
}

var iter = asyncg();

iter.next().then(function(result) {

  // throw will call sync generator prototype built-in function throw
  iter.throw(thrownError).then(
    function(result) {
      throw new Test262Error('throw should cause rejection of promise');
    },
    function(err) {
      assert.sameValue(err, thrownError, "promise should be reject with custom error, got: " + err)

      iter.next().then(({ done, value }) => {
        assert.sameValue(done, true, 'the iterator is completed');
        assert.sameValue(value, undefined, 'value is undefined');
      }).then($DONE, $DONE);
    }
  ).catch($DONE);

}).catch($DONE);
