// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise.all with 2-element array
es6id: S25.4.4.1_A8.2_T2
author: Sam Mikes
description: Promise.all() rejects when second promise in array rejects
flags: [async]
---*/

var rejectP2,
  p1 = Promise.resolve(1),
  p2 = new Promise(function(resolve, reject) {
    rejectP2 = reject;
  });

Promise.all([p1, p2]).then(function() {
  throw new Test262Error("Did not expect promise to be fulfilled.");
}, function(rejected) {
  assert.sameValue(rejected, 2, 'The value of rejected is expected to be 2');
}).then($DONE, $DONE);

rejectP2(2);
