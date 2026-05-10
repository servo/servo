// Copyright 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-throw
description: throw() results in rejected promise when called on completed iterator
info: |
  AsyncGenerator.prototype.throw ( exception )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: throw, [[Value]]: exception, [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  8. If state is not "executing", then
    a. Perform ! AsyncGeneratorResumeNext(generator).
  ...

  AsyncGeneratorResumeNext:
  If completion.[[Type]] is throw, and generator.[[AsyncGeneratorState]] is
  "completed", the resulting promise is rejected with the error.
flags: [async]
features: [async-iteration]
---*/

var throwError = new Error('Catch me');
var g = async function*() {};

var iter = g();
iter.next().then(function(result) {
  assert.sameValue(result.value, undefined);
  assert.sameValue(result.done, true);

  iter.throw(throwError).then($DONE, function(err) {
    assert.sameValue(err, throwError)
  }).then($DONE, $DONE);

}).catch($DONE);
