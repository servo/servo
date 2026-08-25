// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
   Misc sequencing tests
   inspired by https://github.com/promises-aplus/promises-tests/issues/61
   Case "T2a"
es6id: S25.4.4_A2.1_T2
author: Sam Mikes
description: Promise onResolved functions are called in predictable sequence
includes: [promiseHelper.js]
flags: [async]
---*/

var resolveP1, rejectP2, p1, p2,
  sequence = [];

p1 = new Promise(function(resolve, reject) {
  resolveP1 = resolve;
});
p2 = new Promise(function(resolve, reject) {
  rejectP2 = reject;
});

rejectP2(3);
resolveP1(2);

p1.then(function(msg) {
  sequence.push(msg);
});

p2.catch(function(msg) {
  sequence.push(msg);
}).then(function() {
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, "Expected 1,2,3");
}).then($DONE, $DONE);

sequence.push(1);
