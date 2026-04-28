// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.1_A8.1_T1
author: Sam Mikes
description: Promise.all([p1, p2]) resolution functions are called in predictable sequence
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var p1 = new Promise(function(resolve) {
  resolve(1);
});
var p2 = new Promise(function(resolve) {
  resolve(2);
});

sequence.push(1);

p1.then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, "Expected to be called first.");
}).catch($DONE);

Promise.all([p1, p2]).then(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, "Expected to be called third.");
}).then($DONE, $DONE);

p2.then(function() {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, "Expected to be called second.");
}).catch($DONE);

sequence.push(2);
