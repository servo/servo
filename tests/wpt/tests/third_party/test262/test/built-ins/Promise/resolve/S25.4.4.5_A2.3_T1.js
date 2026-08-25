// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.5_A2.3_T1
author: Sam Mikes
description: Promise.resolve passes through an unsettled promise w/ same Constructor
flags: [async]
---*/

var rejectP1,
  p1 = new Promise(function(resolve, reject) {
    rejectP1 = reject;
  }),
  p2 = Promise.resolve(p1),
  arg = {};

assert.sameValue(p1, p2, 'The value of p1 is expected to equal the value of p2');

p2.then(function() {
  throw new Test262Error("Expected p2 to be rejected, not fulfilled.");
}, function(result) {
  assert.sameValue(result, arg, 'The value of result is expected to equal the value of arg');
}).then($DONE, $DONE);

rejectP1(arg);
