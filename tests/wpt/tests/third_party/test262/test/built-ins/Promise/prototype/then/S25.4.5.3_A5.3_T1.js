// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    PerformPromiseThen
    Ref 25.4.5.3.1
es6id: S25.4.5.3_A5.3_T1
author: Sam Mikes
description: Promise.prototype.then immediately queues handler if rejected
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [],
  pReject,
  p = new Promise(function(resolve, reject) {
    pReject = reject;
  });

sequence.push(1);

pReject();

p.then(function() {
  throw new Test262Error("Should not be called -- Promise rejected.");
}, function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3, 'The value of sequence.length is expected to be 3');
  checkSequence(sequence, "Should be first");
}).catch($DONE);

Promise.resolve().then(function() {
  // enqueue another then-handler
  p.then(function() {
    throw new Test262Error("Should not be called (2) -- Promise rejected.");
  }, function() {
    sequence.push(5);
    assert.sameValue(sequence.length, 5, 'The value of sequence.length is expected to be 5');
  checkSequence(sequence, "Should be third");
  }).then($DONE, $DONE);

  sequence.push(4);
  assert.sameValue(sequence.length, 4, 'The value of sequence.length is expected to be 4');
  checkSequence(sequence, "Should be second");
}).catch($DONE);

sequence.push(2);
