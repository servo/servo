// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.3_A7.1_T3
author: Sam Mikes
description: Promise.race([p1, p2]) settles when first settles
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var p1 = new Promise(function() {}),
  p2 = Promise.resolve(2),
  p = Promise.race([p1, p2]);

sequence.push(1);

p.then(function(result) {
  assert.sameValue(result, 2, 'The value of result is expected to be 2');

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
