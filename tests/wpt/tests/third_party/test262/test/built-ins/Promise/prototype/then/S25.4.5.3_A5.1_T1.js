// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    PerformPromiseThen
    Ref 25.4.5.3.1
es6id: S25.4.5.3_A5.1_T1
author: Sam Mikes
description: Promise.prototype.then enqueues handler if pending
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [],
  pResolve,
  p = new Promise(function(resolve, reject) {
    pResolve = resolve;
  });

sequence.push(1);

p.then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, "Should be second");
}).catch($DONE);

Promise.resolve().then(function() {
  // enqueue another then-handler
  p.then(function() {
    sequence.push(4);
    assert.sameValue(sequence.length, 4);
  checkSequence(sequence, "Should be third");
  }).then($DONE, $DONE);

  sequence.push(2);
  assert.sameValue(sequence.length, 2);
  checkSequence(sequence, "Should be first");

  pResolve();
}).catch($DONE);
