// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.3_A6.2_T1
author: Sam Mikes
description: Promise.race([p1]) settles immediately
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var p1 = Promise.reject(1),
  p = Promise.race([p1]);

sequence.push(1);

p.then(function() {
  throw new Test262Error("Should not fulfill.");
}, function() {
  sequence.push(4);
  assert.sameValue(sequence.length, 4, 'The value of sequence.length is expected to be 4');
checkSequence(sequence, "This happens second");
}).catch($DONE);

Promise.resolve().then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3, 'The value of sequence.length is expected to be 3');
  checkSequence(sequence, "This happens first");
}).then(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5, 'The value of sequence.length is expected to be 5');
  checkSequence(sequence, "This happens third");
}).then($DONE, $DONE);

sequence.push(2);
