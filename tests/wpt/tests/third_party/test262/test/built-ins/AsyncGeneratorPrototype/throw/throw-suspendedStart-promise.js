// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-asyncgenerator-prototype-throw
description: >
  Generator is not resumed after a throw completion with a promise arg before
  start
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
  "suspendedStart", generator is closed without being resumed.

  AsyncGeneratorReject will not unwrap Promise values
flags: [async]
features: [async-iteration]
---*/

var g = async function*() {
  throw new Test262Error('Generator must not be resumed.');
};

var it = g();
var promise = new Promise(function() {});

it.throw(promise).then($DONE, function(err) {
  assert.sameValue(err, promise, 'AsyncGeneratorReject(generator, completion.[[Value]])');

  it.next().then(function(ret) {
    assert.sameValue(ret.value, undefined, 'Generator is closed');
    assert.sameValue(ret.done, true, 'Generator is closed');
  }).then($DONE, $DONE);
}).catch($DONE);
