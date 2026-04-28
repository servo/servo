// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise.all with 2-element array
es6id: S25.4.4.1_A8.1_T1
author: Sam Mikes
description: Promise.all() rejects when a promise in its array rejects
flags: [async]
---*/

var rejectP1,
  p1 = new Promise(function(resolve, reject) {
    rejectP1 = reject;
  }),
  p2 = Promise.resolve(2);

Promise.all([p1, p2]).then(function(resolve) {
  throw new Test262Error("Did not expect promise to be fulfilled.");
}, function(rejected) {
  assert.sameValue(rejected, 1, 'The value of rejected is expected to be 1');
}).then($DONE, $DONE);

rejectP1(1);
