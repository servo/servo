// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-asyncgenerator-prototype-return
description: >
  Generator is not resumed after a return type completion.
  Returning non-promise before start
info: |
  AsyncGenerator.prototype.return ( value )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: return, [[Value]]: value, [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  8. If state is not "executing", then
    a. Perform ! AsyncGeneratorResumeNext(generator).
  ...

  AsyncGeneratorResumeNext:
  If completion.[[Type]] is return, and generator.[[AsyncGeneratorState]] is
  "suspendedStart", generator is closed without being resumed.
flags: [async]
features: [async-iteration]
---*/

var g = async function*() {
  throw new Test262Error('Generator must not be resumed.');
};

var it = g();
it.return('sent-value').then(function(ret) {
  assert.sameValue(ret.value, 'sent-value', 'AsyncGeneratorResolve(generator, completion.[[Value]], true)');
  assert.sameValue(ret.done, true, 'AsyncGeneratorResolve(generator, completion.[[Value]], true)');

  it.next().then(function(ret) {
    assert.sameValue(ret.value, undefined, 'Generator is closed');
    assert.sameValue(ret.done, true, 'Generator is closed');
  }).then($DONE, $DONE);
}).catch($DONE);
