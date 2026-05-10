// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
   Misc sequencing tests
   inspired by https://github.com/getify/native-promise-only/issues/34#issuecomment-54282002
es6id: S25.4.2.1_A3.2_T2
author: Sam Mikes
description: Promise onResolved functions are called in predictable sequence
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var p = new Promise(function(resolve, reject) {
  sequence.push(1);
  resolve("");
});

p.then(function() {
  sequence.push(3);
}).then(function() {
  sequence.push(5);
}).then(function() {
  sequence.push(7);
});

p.then(function() {
  sequence.push(4);
}).then(function() {
  sequence.push(6);
}).then(function() {
  sequence.push(8);
}).then(function() {
  assert.sameValue(sequence.length, 8);
  checkSequence(sequence, "Sequence should be as expected");
}).then($DONE, $DONE);

sequence.push(2);
