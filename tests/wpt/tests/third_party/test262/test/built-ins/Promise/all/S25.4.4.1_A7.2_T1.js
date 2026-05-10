// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise.all with 1-element array
    should accept an array with settled promise
es6id: S25.4.4.1_A7.2_T1
author: Sam Mikes
description: Promise.all() accepts a one-element array
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var p1 = new Promise(function(resolve) {
  resolve({});
});

sequence.push(1);

Promise.all([p1]).then(function(resolved) {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, "Expected Promise.all().then to queue second");
}).catch($DONE);

p1.then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, "Expected p1.then to queue first");
}).then(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, "Expected final then to queue last");
}).then($DONE, $DONE);

sequence.push(2);
