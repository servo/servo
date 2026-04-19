// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-asyncgenerator-prototype-throw
description: >
  Thrown generator suspended in a yield position resumes execution within
  the associated catch-block
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
  "suspendedYield", and generator is resumed within a try-block with an
  associated catch block, resume execution within catch-block.
flags: [async]
features: [async-iteration]
---*/

var error = new Error('boop');
var g = async function*() {
  try {
    yield 1;
    throw new Test262Error('Generator must be resumed in catch block.');
  } catch (err) {
    assert.sameValue(err, error);
    return 'done';
  }
};

var it = g();
it.next().then(function(ret) {
  assert.sameValue(ret.value, 1, 'Initial yield');
  assert.sameValue(ret.done, false, 'Initial yield');

  it.throw(error).then(function(ret) {
    assert.sameValue(ret.value, 'done', 'AsyncGeneratorResolve(generator, resultValue, true)');
    assert.sameValue(ret.done, true, 'AsyncGeneratorResolve(generator, resultValue, true)');

    it.next().then(function(ret) {
      assert.sameValue(ret.value, undefined, 'Generator is closed');
      assert.sameValue(ret.done, true, 'Generator is closed');
    }).then($DONE, $DONE);

  }).catch($DONE);

}).catch($DONE);
